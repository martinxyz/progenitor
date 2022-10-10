use hex2d::Angle;
use hex2d::Direction;
use hex2d::Spin;
use nalgebra::SVector;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_distr::Normal;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::AxialTile;
use crate::CellView;
use crate::Simulation;

// use super::builder_agent::Agent;
use super::nn;

#[derive(Serialize, Deserialize)]
struct Builder {
    pos: coords::Cube,
    heading: Direction,
}

#[derive(PartialEq)]
enum Action {
    Forward,
    Left,
    Right,
    Pullback,
}

impl From<usize> for Action {
    fn from(idx: usize) -> Self {
        match idx {
            0 => Action::Forward,
            1 => Action::Left,
            2 => Action::Right,
            3 => Action::Pullback,
            _ => panic!("invalid action choice"),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
enum Cell {
    Air,
    Stone,
    Border,
}

#[derive(Serialize, Deserialize)]
struct State {
    cells: AxialTile<Cell>,
    visited: AxialTile<bool>,
    builders: Vec<Builder>,
    rng: rand_pcg::Lcg64Xsh32,
}

pub struct Builders {
    state: State,
    // agent: Agent,
    nn: nn::Network,
    max_depth_reached: i32,
}

const TILE_WIDTH: i32 = 40;
const TILE_HEIGHT: i32 = 40;

const CENTER: coords::Offset = coords::Offset {
    // would be easier in axial coordinates...
    col: (2 * TILE_WIDTH + TILE_HEIGHT) / 4,
    row: TILE_HEIGHT / 2,
};

impl Builders {
    pub fn new_optimized() -> Builders {
        Self::new_with_params(&super::optimized_params::PARAMS)
    }

    pub const PARAM_COUNT: usize = nn::PARAM_COUNT;

    pub fn new_with_random_params() -> Builders {
        let rng = &mut thread_rng();
        let dist = Normal::new(0.0, 1.0).unwrap();
        let params: SVector<f32, { Self::PARAM_COUNT }> = SVector::from_distribution(&dist, rng);
        Self::new_with_params(&params.into())
    }

    pub fn new_with_params(params: &[f32; Self::PARAM_COUNT]) -> Builders {
        let nn = nn::Network::new(params);
        let seed = thread_rng().next_u64();
        let mut rng: rand_pcg::Lcg64Xsh32 = Pcg32::seed_from_u64(seed);

        let center: coords::Cube = CENTER.into();
        let create_builder = |_| Builder {
            pos: center + *Direction::all().choose(&mut rng).unwrap(),
            heading: *Direction::all().choose(&mut rng).unwrap(),
        };
        let mut cells = AxialTile::new(TILE_WIDTH, TILE_HEIGHT, Cell::Border);
        for radius in 0..18 {
            for pos in center.ring_iter(radius, Spin::CCW(Direction::XY)) {
                cells.set_cell(
                    pos,
                    match radius {
                        0..=6 => Cell::Air,
                        _ => Cell::Stone,
                    },
                );
            }
        }
        Builders {
            // agent,
            nn,
            state: State {
                visited: AxialTile::new(TILE_WIDTH, TILE_HEIGHT, false),
                cells,
                builders: (0..5).map(create_builder).collect(),
                rng,
            },
            max_depth_reached: 0,
        }
    }

    pub fn avg_visited(&self) -> f32 {
        let total = self.state.visited.area();
        let visited: i32 = self.state.visited.iter_cells().map(|&v| i32::from(v)).sum();
        visited as f32 / total as f32
    }

    pub fn score(&self) -> f32 {
        self.max_depth_reached as f32 // + self.avg_visited()
    }

    pub fn print_stats(&self) {
        self.nn.print_stats();
    }
}

impl Simulation for Builders {
    fn step(&mut self) {
        for t in self.state.builders.iter_mut() {
            // let action = self.agent.act(&mut self.state.rng).into();
            let look = |angle: Angle| {
                let not_air = self
                    .state
                    .cells
                    .get_cell(t.pos + (t.heading + angle))
                    .map(|cell| cell != Cell::Air)
                    .unwrap_or(false);
                if not_air {
                    1.0
                } else {
                    0.0
                }
            };
            let inputs = [
                look(Angle::Forward),
                look(Angle::Left),
                look(Angle::Right),
                look(Angle::LeftBack),
                look(Angle::RightBack),
                look(Angle::Back),
            ];

            let outputs: SVector<f32, 4> = self.nn.forward(inputs);
            let action = nn::softmax_choice(outputs, &mut self.state.rng).into();

            let turn = match action {
                Action::Left => Angle::Left,
                Action::Right => Angle::Right,
                _ => Angle::Forward,
            };
            t.heading = t.heading + turn;

            let pos_forward = t.pos + t.heading;
            let pos_back = t.pos - t.heading;

            if let Some(cell_forward) = self.state.cells.get_cell(pos_forward) {
                if action != Action::Pullback {
                    if cell_forward == Cell::Air {
                        t.pos = pos_forward;
                    }
                } else if let Some(cell_back) = self.state.cells.get_cell(pos_back) {
                    if cell_back == Cell::Air {
                        let cell_here = self.state.cells.get_cell(t.pos).unwrap();
                        self.state.cells.set_cell(pos_back, cell_here);
                        self.state.cells.set_cell(t.pos, cell_forward);
                        self.state.cells.set_cell(pos_forward, Cell::Air);
                        t.pos = pos_back;
                    }
                }
            }
            self.state.visited.set_cell(t.pos, true);
            let center: coords::Cube = CENTER.into();
            self.max_depth_reached = self.max_depth_reached.max(center.distance(t.pos));
        }
    }

    fn get_cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        let pos = coords::Cube {
            // hack to translate web UI the viewport a bit
            x: pos.x + 12,
            y: pos.y - 12,
        };
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

        let energy: u8 = self.state.visited.get_cell(pos)?.into();
        let cell_type = match self.state.cells.get_cell(pos)? {
            Cell::Air => 2,
            Cell::Stone => 4,
            Cell::Border => 255,
        };
        Some(CellView {
            cell_type,
            energy: Some(energy),
            ..Default::default()
        })
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self.state).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        self.state = bincode::deserialize_from(data).unwrap();
    }
}
