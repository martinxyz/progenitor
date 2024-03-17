use hex2d::Angle;
use nalgebra::SVector;
use num_traits::FromPrimitive;
use rand::distributions;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_distr::Distribution;
use rand_distr::Normal;
use serde::{Deserialize, Serialize};

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
    last_action: Action,
    memory: SVector<f32, N_MEMORY>,
}

const N_ACTIONS: usize = 10;
const N_MEMORY: usize = 4;

#[derive(Clone, Copy, PartialEq, FromPrimitive, Serialize, Deserialize)]
enum Action {
    Forward,
    ForwardLeft,
    ForwardRight,
    Back,
    BackLeft,
    BackRight,
    PullBack,
    PullBackLeft,
    PullBackRight,
    Wait,
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Cell {
    Border,
    Air,
    Wall,
    Agent,
    Food,
}
use Cell::*;

impl Cell {
    fn can_pull(self) -> bool {
        matches!(self, Wall | Food | Air)
    }
    fn can_walk(self) -> bool {
        matches!(self, Food | Air)
    }
}

fn cost(action: Action, moved: Cell) -> u8 {
    let move_cost = match moved {
        Air => 0,
        Food => 0,
        Wall => 2,
        Agent => unreachable!(),
        Border => unreachable!(),
    };
    use Action::*;
    let action_cost = match action {
        Forward | ForwardLeft | ForwardRight => 0,
        Back | BackLeft | BackRight => 1,
        PullBack | PullBackLeft | PullBackRight => 1,
        Wait => 0,
    };
    move_cost + action_cost
}

#[derive(Serialize, Deserialize)]
struct State {
    cells: AxialTile<Cell>,
    visited: AxialTile<bool>,
    builders: Vec<Builder>,
    rng: SimRng,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Params {
    pub builder_hyperparams: nn::Hyperparams,
    pub builder_weights: Box<[f32]>,
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

    pub fn new_with_random_params() -> Builders {
        let hp = nn::Hyperparams {
            n_hidden: 10,
            n_hidden2: 10,
            init_fac: 1.0,
            bias_fac: 0.1,
        };
        let rng = &mut thread_rng();
        let dist = Normal::new(0.0, 1.0).unwrap();
        let weights = dist.sample_iter(rng).take(hp.count_params()).collect();

        Self::new_with_params(Params {
            builder_hyperparams: hp,
            builder_weights: weights,
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
            last_action: Action::Wait,
        });
    }

    fn move_builders(&mut self) {
        for t in self.state.builders.iter_mut() {
            if t.exhausted > 0 {
                t.exhausted -= 1;
                continue;
            }
            let look = |item: Cell, angle: Angle| {
                if self.state.cells.cell_unchecked(t.pos + (t.heading + angle)) == item {
                    1.0
                } else {
                    0.0
                }
            };
            let last_action_was = |action: Action| {
                if t.last_action == action {
                    1.0
                } else {
                    0.0
                }
            };
            let builders_nearby: i32 = self
                .state
                .cells
                .neighbours_unchecked(t.pos)
                .map(|(_, cell)| (cell == Cell::Agent) as i32)
                .iter()
                .sum();

            let inputs = [
                look(Air, Angle::Forward),
                look(Air, Angle::Left),
                look(Air, Angle::Right),
                look(Air, Angle::LeftBack),
                look(Air, Angle::RightBack),
                look(Air, Angle::Back),
                look(Food, Angle::Forward),
                look(Food, Angle::Left),
                look(Food, Angle::Right),
                look(Food, Angle::LeftBack),
                look(Food, Angle::RightBack),
                look(Food, Angle::Back),
                look(Wall, Angle::Forward),
                look(Wall, Angle::Left),
                look(Wall, Angle::Right),
                look(Wall, Angle::LeftBack),
                look(Wall, Angle::RightBack),
                look(Wall, Angle::Back),
                look(Agent, Angle::Forward),
                builders_nearby as f32 * 10.,
                // ... maybe give them a "dist from center" sensor? (aka gradient back to hive)
                last_action_was(Action::Forward),
                last_action_was(Action::ForwardLeft),
                last_action_was(Action::ForwardRight),
                last_action_was(Action::Back),
                last_action_was(Action::BackLeft),
                last_action_was(Action::BackRight),
                last_action_was(Action::PullBack),
                last_action_was(Action::PullBackLeft),
                last_action_was(Action::PullBackRight),
                last_action_was(Action::Wait),
                t.memory[0],
                t.memory[1],
                t.memory[2],
                t.memory[3],
            ]
            .map(|x| (x - 0.2) * 2.5); // input normalization for minimalists
            assert_eq!(N_MEMORY, 4);
            self.encounters += builders_nearby;

            let outputs: SVector<f32, { nn::N_OUTPUTS }> = self.nn.forward(inputs);
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
            t.last_action = action;

            let mut moved = Air;

            use Action::*;
            if action != Action::Wait {
                let (step, turn) = match action {
                    Forward => (Angle::Forward, Angle::Forward),
                    ForwardLeft => (Angle::Left, Angle::Left),
                    ForwardRight => (Angle::Right, Angle::Right),
                    Back | PullBack => (Angle::Back, Angle::Forward),
                    BackLeft | PullBackLeft => (Angle::LeftBack, Angle::Right),
                    BackRight | PullBackRight => (Angle::RightBack, Angle::Left),
                    Wait => unreachable!(),
                };

                let pos_old = t.pos;
                let pos_forward = pos_old + t.heading;
                let pos_step = pos_old + (t.heading + step);
                t.heading = t.heading + turn;

                let cell_forward = self.state.cells.cell_unchecked(pos_forward);
                let cell_step = self.state.cells.cell_unchecked(pos_step);
                match action {
                    Forward | ForwardLeft | ForwardRight | Back | BackLeft | BackRight => {
                        if cell_step.can_walk() {
                            moved = cell_step;
                            self.state.cells.set_cell(pos_old, cell_step);
                            self.state.cells.set_cell(pos_step, Agent);
                            t.pos = pos_step;
                        }
                    }
                    PullBack | PullBackRight | PullBackLeft => {
                        if cell_step == Air && cell_forward.can_pull() {
                            moved = cell_forward;
                            self.state.cells.set_cell(pos_forward, Air);
                            self.state.cells.set_cell(pos_old, cell_forward);
                            self.state.cells.set_cell(pos_step, Agent);
                            t.pos = pos_step;
                        } else if cell_step.can_walk() {
                            moved = cell_step;
                            self.state.cells.set_cell(pos_old, cell_step);
                            self.state.cells.set_cell(pos_step, Agent);
                            t.pos = pos_step;
                        }
                    }
                    Wait => unreachable!(),
                }
            }

            t.exhausted += cost(action, moved);

            self.state.visited.set_cell(t.pos, true);
            let center: coords::Cube = hexmap::center(RING_RADIUS);

            if let Some(nh) = self.state.cells.neighbourhood(t.pos) {
                self.walls_nearby += nh.count_neighbours(|n| matches!(n, Wall | Border));
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
        cells.count_edges(|cell| matches!(cell, Wall)) as f32 / cells.area() as f32
    }

    pub fn hoarding_score(&self) -> i32 {
        let mut count = 0;
        for nh in self.state.cells.iter_valid_neighbourhoods() {
            if nh.center == Food {
                count += nh.count_neighbours(|c| c == Food).pow(2)
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
            Border => return None,
            Air => 2,
            Wall => 4,
            Agent => 0,
            Food => 1,
        };
        Some(CellView {
            cell_type,
            ..Default::default()
        })
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        hexmap::viewport(RING_RADIUS)
    }
}
