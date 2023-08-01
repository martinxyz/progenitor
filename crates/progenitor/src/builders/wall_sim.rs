use hex2d::Angle;
use hex2d::Coordinate;
use nalgebra::SVector;
use rand::distributions;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_distr::Normal;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::coords;
use crate::coords::Direction;
use crate::hexmap;
use crate::AxialTile;
use crate::CellView;
use crate::HexgridView;
use crate::SimRng;
use crate::Simulation;

use super::nn;
use super::optimized_params;
use super::worldgen;
use super::worldgen::RING_RADIUS;

#[derive(Serialize, Deserialize)]
struct Builder {
    pos: coords::Cube,
    heading: Direction,
    exhausted: u8,
    memory: SVector<f32, 4>,
}

#[derive(PartialEq)]
enum Action {
    Forward,
    Left,
    Right,
    Pull,
}

impl From<usize> for Action {
    fn from(idx: usize) -> Self {
        match idx {
            0 => Action::Forward,
            1 => Action::Left,
            2 => Action::Right,
            3 => Action::Pull,
            _ => panic!("invalid action choice"),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Cell {
    Border,
    Air,
    Wall,
    Builder,
    Food,
}

impl Cell {
    fn move_cost(self) -> u8 {
        match self {
            Cell::Wall => 6,
            Cell::Food => 2,
            _ => 255,
        }
    }
    fn can_move(self) -> bool {
        self.move_cost() < 255
    }
}

#[derive(Serialize, Deserialize)]
struct State {
    cells: AxialTile<Cell>,
    visited: AxialTile<bool>,
    mass: AxialTile<u8>,
    builders: Vec<Builder>,
    rng: SimRng,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Params {
    #[serde(with = "BigArray")]
    pub builder_weights: [f32; nn::PARAM_COUNT],
    pub builder_hyperparams: nn::Hyperparams,
    pub memory_clamp: f32,
    pub memory_halftime: f32,
    pub actions_scale: f32,
}

pub struct Builders {
    state: State,
    nn: nn::Network,
    // our own hyperparams
    memory_decay: f32,
    memory_clamp: f32,
    actions_scale: f32,
    // for score or BCs:
    pub max_depth_reached: i32,
    pub encounters: i32,
}

impl Simulation for Builders {
    fn step(&mut self) {
        let builder_positions: Vec<_> = self.state.builders.iter().map(|t| t.pos).collect();
        for pos in builder_positions {
            self.kick_dust(pos);
        }
        self.move_builders();
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self.state).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        self.state = bincode::deserialize_from(data).unwrap();
    }
}

impl Builders {
    pub fn new_optimized() -> Builders {
        match optimized_params::load() {
            Some(params) => Self::new_with_params(params),
            None => Self::new_with_random_params(),
        }
    }

    pub const PARAM_COUNT: usize = nn::PARAM_COUNT;

    pub fn new_with_random_params() -> Builders {
        let rng = &mut thread_rng();
        let dist = Normal::new(0.0, 1.0).unwrap();
        let weights: SVector<f32, { Self::PARAM_COUNT }> = SVector::from_distribution(&dist, rng);
        Self::new_with_params(Params {
            builder_weights: weights.into(),
            builder_hyperparams: nn::Hyperparams {
                init_fac: 1.0,
                bias_fac: 0.1,
            },
            memory_halftime: 20.0,
            memory_clamp: 5.0,
            actions_scale: 3.0,
        })
    }

    pub fn new_with_params(params: Params) -> Builders {
        let seed = thread_rng().next_u64();
        Self::new_with_params_and_seed(params, seed)
    }

    pub fn new_with_params_and_seed(params: Params, seed: u64) -> Builders {
        let nn = nn::Network::new(&params.builder_weights, params.builder_hyperparams);
        let mut rng = SimRng::seed_from_u64(seed);

        let cells = worldgen::create_world(&mut rng);

        let mut builders = Builders {
            nn,
            state: State {
                visited: hexmap::new(RING_RADIUS, false, |_| false),
                cells,
                mass: hexmap::new(RING_RADIUS, 2, |_| 2),
                builders: Vec::new(),
                rng,
            },
            max_depth_reached: 0,
            encounters: 0,
            memory_decay: f32::ln(2.) / params.memory_halftime.max(0.001),
            memory_clamp: params.memory_clamp.max(0.001),
            actions_scale: params.actions_scale,
        };

        for _ in 0..5 {
            builders.add_builder();
        }

        builders
    }

    fn add_builder(&mut self) {
        let rng = &mut self.state.rng;
        let pos = worldgen::find_agent_starting_place(rng, &self.state.cells);
        self.state.cells.set_cell(pos, Cell::Builder);
        let memory = {
            let dist = distributions::Uniform::new(-1.0f32, 1.0f32);
            SVector::from_distribution(&dist, rng)
        };
        self.state.builders.push(Builder {
            pos,
            heading: *Direction::all().choose(rng).unwrap(),
            exhausted: 0,
            memory,
        });
    }

    fn kick_dust(&mut self, pos: Coordinate) {
        let rng = &mut self.state.rng;
        let dir = *Direction::all().choose(rng).unwrap();
        if let (Some(Cell::Builder), Some(Cell::Air)) =
            (self.state.cells.cell(pos), self.state.cells.cell(pos + dir))
        {
            if let (Some(src), Some(dst)) =
                (self.state.mass.cell(pos), self.state.mass.cell(pos + dir))
            {
                if src > 0 && dst < 255 {
                    self.state.mass.set_cell(pos, src - 1);
                    self.state.mass.set_cell(pos + dir, dst + 1);
                }
            }
        }
    }

    fn move_builders(&mut self) {
        for t in self.state.builders.iter_mut() {
            if t.exhausted > 0 {
                t.exhausted -= 1;
                if t.exhausted > 8 {
                    continue; // rest
                }
            }
            // let action = self.agent.act(&mut self.state.rng).into();
            let look = |item: Cell, angle: Angle| {
                let present = self
                    .state
                    .cells
                    .cell(t.pos + (t.heading + angle))
                    .map(|c| c == item)
                    .unwrap_or(false);
                if present {
                    1.0
                } else {
                    0.0
                }
            };
            let look_far = |item: Cell, angle: Angle| {
                let present = self
                    .state
                    .cells
                    .cell(t.pos + (t.heading + angle) + (t.heading + angle))
                    .map(|c| c == item)
                    .unwrap_or(false);
                if present {
                    1.0
                } else {
                    0.0
                }
            };
            let builders_nearby: i32 = self
                .state
                .cells
                .neighbours(t.pos)
                .map(|(_, cell)| (cell == Some(Cell::Builder)) as i32)
                .iter()
                .sum();

            let dust_here = self.state.mass.cell(t.pos).unwrap_or(0).into();
            let dust_nearby = self
                .state
                .mass
                .neighbourhood(t.pos)
                .unwrap()
                .neighbours
                .into_iter()
                .map(|mass| mass as f32)
                .sum::<f32>()
                * 0.1;

            let inputs = [
                look(Cell::Air, Angle::Forward),
                look(Cell::Air, Angle::Left),
                look(Cell::Air, Angle::Right),
                look(Cell::Air, Angle::LeftBack),
                look(Cell::Air, Angle::RightBack),
                look(Cell::Air, Angle::Back),
                look_far(Cell::Air, Angle::Forward),
                look(Cell::Food, Angle::Forward),
                look(Cell::Food, Angle::Left),
                look(Cell::Food, Angle::Right),
                look(Cell::Food, Angle::LeftBack),
                look(Cell::Food, Angle::RightBack),
                look(Cell::Food, Angle::Back),
                look_far(Cell::Food, Angle::Forward),
                look(Cell::Builder, Angle::Forward),
                dust_here,
                dust_nearby,
                builders_nearby as f32 * 10.,
                t.memory[0],
                t.memory[1],
                t.memory[2],
                t.memory[3],
            ];
            self.encounters += builders_nearby;

            let outputs: SVector<f32, 8> = self.nn.forward(inputs);
            let mut action_logits = outputs.fixed_rows::<4>(0).clone_owned();
            let memory_update = outputs.fixed_rows::<4>(4).clone_owned();
            t.memory *= self.memory_decay;
            t.memory += (1. - self.memory_decay) * memory_update;
            t.memory = nalgebra::clamp(
                t.memory,
                SVector::from_element(-self.memory_clamp),
                SVector::from_element(self.memory_clamp),
            );

            action_logits.apply(|v| *v *= self.actions_scale);
            let action = nn::softmax_choice(action_logits, &mut self.state.rng).into();

            let turn = match action {
                Action::Left => Angle::Left,
                Action::Right => Angle::Right,
                _ => Angle::Forward,
            };
            t.heading = t.heading + turn;

            let pos_forward = t.pos + t.heading;
            let pos_back = t.pos - t.heading;

            if let Some(cell_forward) = self.state.cells.cell(pos_forward) {
                match action {
                    Action::Pull => {
                        if let Some(cell_back) = self.state.cells.cell(pos_back) {
                            if cell_back == Cell::Air {
                                if cell_forward.can_move() {
                                    t.exhausted += cell_forward.move_cost();
                                    self.state.cells.set_cell(pos_forward, Cell::Air);
                                    self.state.cells.set_cell(t.pos, cell_forward);
                                } else {
                                    self.state.cells.set_cell(t.pos, Cell::Air);
                                }
                                t.pos = pos_back;
                                self.state.cells.set_cell(t.pos, Cell::Builder);
                            }
                        }
                    }
                    Action::Forward => {
                        if cell_forward.can_move() {
                            // push
                            let pos_forward2x = pos_forward + t.heading;
                            if let Some(cell_forward2x) = self.state.cells.cell(pos_forward2x) {
                                if cell_forward2x == Cell::Air {
                                    t.exhausted += cell_forward.move_cost();
                                    self.state.cells.set_cell(pos_forward2x, cell_forward);
                                    self.state.cells.set_cell(t.pos, Cell::Air);
                                    t.pos = pos_forward;
                                    self.state.cells.set_cell(t.pos, Cell::Builder);
                                }
                            }
                        } else if cell_forward == Cell::Air {
                            self.state.cells.set_cell(t.pos, Cell::Air);
                            t.pos = pos_forward;
                            self.state.cells.set_cell(t.pos, Cell::Builder);
                        }
                    }
                    Action::Left | Action::Right => {
                        if cell_forward == Cell::Air {
                            self.state.cells.set_cell(t.pos, Cell::Air);
                            t.pos = pos_forward;
                            self.state.cells.set_cell(t.pos, Cell::Builder);
                        }
                    }
                }
            }
            self.state.visited.set_cell(t.pos, true);
            let center: coords::Cube = hexmap::center(RING_RADIUS);
            self.max_depth_reached = self.max_depth_reached.max(center.distance(t.pos));
        }
    }

    pub fn avg_visited(&self) -> f32 {
        let total = self.state.visited.area();
        let visited: i32 = self.state.visited.iter_cells().map(|&v| i32::from(v)).sum();
        visited as f32 / total as f32
    }

    pub fn encounters(&self) -> i32 {
        self.encounters
    }

    pub fn max_depth_reached(&self) -> i32 {
        self.max_depth_reached
    }

    pub fn relative_wall_edges(&self) -> f32 {
        let cells = &self.state.cells;
        cells.count_edges(|cell| matches!(cell, Cell::Wall)) as f32 / cells.area() as f32
    }

    pub fn hoarding_score(&self) -> i32 {
        let mut count = 0;
        for nh in self.state.cells.iter_valid_neighbourhoods() {
            if nh.center == Cell::Food {
                count += nh.count_neighbours(|c| c == Cell::Food).pow(2)
            }
        }
        count
    }

    pub fn print_stats(&self) {
        self.nn.print_stats();
    }
}

impl HexgridView for Builders {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        // xxx inefficient when this gets called for all cells...
        for t in self.state.builders.iter() {
            if pos == t.pos {
                return Some(CellView {
                    cell_type: 0,
                    direction: Some(t.heading),
                    ..Default::default()
                });
            }
        }

        let cell_type = match self.state.cells.cell(pos)? {
            Cell::Border => return None,
            Cell::Air => 2,
            Cell::Wall => 4,
            Cell::Builder => 0,
            Cell::Food => 1,
        };
        let energy: u8 = self.state.mass.cell(pos)?;
        Some(CellView {
            cell_type,
            energy: Some(energy),
            ..Default::default()
        })
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        hexmap::viewport(RING_RADIUS)
    }
}
