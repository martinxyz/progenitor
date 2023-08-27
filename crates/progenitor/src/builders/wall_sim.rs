use hex2d::Angle;
use hex2d::Coordinate;
use nalgebra::SVector;
use num_traits::FromPrimitive;
use rand::distributions;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;
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
    memory: SVector<f32, N_MEMORY>,
}

const N_ACTIONS: usize = 9;
const N_MEMORY: usize = 4;

#[derive(PartialEq, FromPrimitive)]
enum Action {
    Forward,
    ForwardLeft,
    ForwardRight,
    CircleLeft,
    CircleRight,
    PullBack,
    PullBackLeft,
    PullBackRight,
    Mark,
}

impl Action {
    fn cost(&self) -> u8 {
        match self {
            Action::Forward => 0,
            Action::ForwardLeft => 0,
            Action::ForwardRight => 0,
            Action::CircleLeft => 1,
            Action::CircleRight => 1,
            Action::PullBack => 2,
            Action::PullBackLeft => 3,
            Action::PullBackRight => 3,
            Action::Mark => 1,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Cell {
    Border,
    Air,
    Wall,
    Agent,
    Food,
}

impl Cell {
    fn move_cost(self) -> u8 {
        match self {
            Cell::Wall => 3,
            Cell::Food => 1,
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
    markers: AxialTile<u8>,
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
    pub walls_nearby: i32,
}

impl Simulation for Builders {
    fn step(&mut self) {
        for builder in &self.state.builders {
            disturb_marker(&mut self.state.markers, &self.state.cells, builder.pos, &mut self.state.rng);
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
                markers: hexmap::new(RING_RADIUS, 2, |_| 2),
                builders: Vec::new(),
                rng,
            },
            max_depth_reached: 0,
            encounters: 0,
            walls_nearby: 0,
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
        self.state.cells.set_cell(pos, Cell::Agent);
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

    fn move_builders(&mut self) {
        for t in self.state.builders.iter_mut() {
            if t.exhausted > 0 {
                t.exhausted -= 1;
                continue;
            }
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
            let look_marker = |angle: Angle| {
                self.state
                    .markers
                    .cell(t.pos + (t.heading + angle))
                    .map(|c| c as f32 * (1. / 8.))
                    .unwrap_or(0.)
            };
            let builders_nearby: i32 = self
                .state
                .cells
                .neighbours(t.pos)
                .map(|(_, cell)| (cell == Some(Cell::Agent)) as i32)
                .iter()
                .sum();

            let marker_here = self.state.markers.cell(t.pos).unwrap() as f32 * (1. / 8.);

            let inputs = [
                look(Cell::Air, Angle::Forward),
                look(Cell::Air, Angle::Left),
                look(Cell::Air, Angle::Right),
                look(Cell::Air, Angle::LeftBack),
                look(Cell::Air, Angle::RightBack),
                look(Cell::Air, Angle::Back),
                look(Cell::Food, Angle::Forward),
                look(Cell::Food, Angle::Left),
                look(Cell::Food, Angle::Right),
                look(Cell::Food, Angle::LeftBack),
                look(Cell::Food, Angle::RightBack),
                look(Cell::Food, Angle::Back),
                look_marker(Angle::Forward),
                look_marker(Angle::Left),
                look_marker(Angle::Right),
                look_marker(Angle::LeftBack),
                look_marker(Angle::RightBack),
                look_marker(Angle::Back),
                look(Cell::Agent, Angle::Forward),
                marker_here,
                builders_nearby as f32 * 10.,
                t.memory[0],
                t.memory[1],
                t.memory[2],
                t.memory[3],
            ];
            assert_eq!(N_MEMORY, 4);
            self.encounters += builders_nearby;

            let outputs: SVector<f32, { nn::N_OUTPUTS }> = self
                .nn
                .forward(inputs.try_into().expect("input count should match"));
            let mut action_logits = outputs.fixed_rows::<N_ACTIONS>(0).clone_owned();
            let memory_update = outputs.fixed_rows::<N_MEMORY>(N_ACTIONS).clone_owned();
            t.memory *= self.memory_decay;
            t.memory += (1. - self.memory_decay) * memory_update;
            t.memory = nalgebra::clamp(
                t.memory,
                SVector::from_element(-self.memory_clamp),
                SVector::from_element(self.memory_clamp),
            );

            action_logits.apply(|v| *v *= self.actions_scale);
            let action = Action::from_usize(nn::softmax_choice(action_logits, &mut self.state.rng))
                .expect("logits should match action count");
            t.exhausted += action.cost();

            let (step, turn) = match action {
                Action::Forward => (Angle::Forward, Angle::Forward),
                Action::ForwardLeft => (Angle::Left, Angle::Left),
                Action::ForwardRight => (Angle::Right, Angle::Right),
                Action::CircleLeft => (Angle::Left, Angle::Right),
                Action::CircleRight => (Angle::Right, Angle::Left),
                Action::PullBack => (Angle::Back, Angle::Forward),
                Action::PullBackLeft => (Angle::LeftBack, Angle::Right),
                Action::PullBackRight => (Angle::RightBack, Angle::Left),
                Action::Mark => (Angle::Forward, Angle::Forward),
            };

            let pos_forward = t.pos + t.heading;
            let pos_step = t.pos + (t.heading + step);
            t.heading = t.heading + turn;

            if action == Action::Mark {
                if let Some(marker_forward) = self.state.markers.cell(pos_forward) {
                    if marker_forward < 16 {
                        self.state.markers.set_cell(pos_forward, 16);
                    }
                }
                if let Some(marker_here) = self.state.markers.cell(t.pos) {
                    if marker_here < 32 {
                        self.state.markers.set_cell(t.pos, marker_here + 4);
                    }
                }
            }

            if self.state.cells.cell(pos_step) == Some(Cell::Air) {
                self.state.cells.set_cell(t.pos, Cell::Air);
                self.state.cells.set_cell(pos_step, Cell::Agent);
                if matches!(
                    action,
                    Action::PullBack | Action::PullBackLeft | Action::PullBackRight
                ) {
                    if let Some(cell_forward) = self.state.cells.cell(pos_forward) {
                        if cell_forward.can_move() {
                            t.exhausted += cell_forward.move_cost();
                            self.state.cells.set_cell(pos_forward, Cell::Air);
                            self.state.cells.set_cell(t.pos, cell_forward);
                        }
                    }
                }
                t.pos = pos_step;
            }

            self.state.visited.set_cell(t.pos, true);
            let center: coords::Cube = hexmap::center(RING_RADIUS);

            if let Some(nh) = self.state.cells.neighbourhood(t.pos) {
                self.walls_nearby +=
                    nh.count_neighbours(|n| matches!(n, Cell::Wall | Cell::Border));
            }
            self.max_depth_reached = self.max_depth_reached.max(center.distance(t.pos));
        }
    }

    pub fn avg_visited(&self) -> f32 {
        let total = self.state.visited.area();
        let visited: i32 = self.state.visited.iter_cells().map(|&v| i32::from(v)).sum();
        visited as f32 / total as f32
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


fn disturb_marker(markers: &mut AxialTile<u8>, cells: &AxialTile<Cell>, pos: Coordinate, rng: &mut impl Rng) {
        let dir = *Direction::all().choose(rng).unwrap();
        if let (Some(Cell::Agent), Some(Cell::Air)) =
            (cells.cell(pos), cells.cell(pos + dir))
        {
            if let (Some(src), Some(dst)) = (
                markers.cell(pos),
                markers.cell(pos + dir),
            ) {
                if src > 0 {
                    markers.set_cell(pos, src - 1);
                    if rng.gen_bool(0.8) {
                        markers.set_cell(pos + dir, dst.saturating_add(1));
                    }
                }
            }
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
            Cell::Agent => 0,
            Cell::Food => 1,
        };
        let energy: u8 = self.state.markers.cell(pos)?;
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
