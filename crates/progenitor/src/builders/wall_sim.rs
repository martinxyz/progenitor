use hex2d::Angle;
use hex2d::Coordinate;
use hex2d::Direction;
use hex2d::Spin;
use nalgebra::SVector;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_distr::Normal;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::coords;
use crate::AxialTile;
use crate::CellView;
use crate::HexgridView;
use crate::Simulation;

use super::nn;
use super::optimized_params;

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
    Floor,
    Stone,
    Border,
    Builder,
}

#[derive(Serialize, Deserialize)]
struct State {
    cells: AxialTile<Cell>,
    visited: AxialTile<bool>,
    mass: AxialTile<u8>,
    builders: Vec<Builder>,
    rng: rand_pcg::Lcg64Xsh32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Params {
    #[serde(with = "BigArray")]
    pub builder_weights: [f32; nn::PARAM_COUNT],
    pub builder_hyperparams: nn::Hyperparams,
}

pub struct Builders {
    state: State,
    nn: nn::Network,
    // for score or BCs:
    pub max_depth_reached: i32,
    pub encounters: i32,
}

// FIXME: Should calculate all this stuff, and also the viewport...
//        But not here. In a new HexagonTile module.
const RING_RADIUS: i32 = 30;
const TILE_WIDTH: i32 = RING_RADIUS * 3;
const TILE_HEIGHT: i32 = RING_RADIUS * 2;

const CENTER: coords::Offset = coords::Offset {
    // would be easier in axial coordinates...
    col: (2 * TILE_WIDTH + TILE_HEIGHT) / 4,
    row: TILE_HEIGHT / 2,
};

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
        })
    }

    pub fn new_with_params(params: Params) -> Builders {
        let nn = nn::Network::new(&params.builder_weights, params.builder_hyperparams);
        let seed = thread_rng().next_u64();
        let mut rng: rand_pcg::Lcg64Xsh32 = Pcg32::seed_from_u64(seed);

        let center: coords::Cube = CENTER.into();
        let mut cells = AxialTile::new(TILE_WIDTH, TILE_HEIGHT, Cell::Border);
        for radius in 0..RING_RADIUS {
            for pos in center.ring_iter(radius, Spin::CCW(Direction::XY)) {
                cells.set_cell(
                    pos,
                    match rng.gen_bool(0.25) {
                        false => Cell::Floor,
                        true => Cell::Stone,
                    },
                );
            }
        }
        let mut builders = Builders {
            nn,
            state: State {
                visited: AxialTile::new(TILE_WIDTH, TILE_HEIGHT, false),
                cells,
                mass: AxialTile::new(TILE_WIDTH, TILE_HEIGHT, 2),
                builders: Vec::new(),
                rng,
            },
            max_depth_reached: 0,
            encounters: 0,
        };

        for _ in 0..5 {
            builders.add_builder(center);
        }

        builders
    }

    fn add_builder(&mut self, pos: Coordinate) {
        let rng = &mut self.state.rng;
        let mut heading = *Direction::all().choose(rng).unwrap();
        let mut pos = pos;
        while self.state.cells.cell(pos) != Some(Cell::Floor) {
            heading = *Direction::all().choose(rng).unwrap();
            pos = pos + heading;
        }
        self.state.cells.set_cell(pos, Cell::Builder);
        self.state.builders.push(Builder { pos, heading });
    }

    fn kick_dust(&mut self, pos: Coordinate) {
        let rng = &mut self.state.rng;
        let dir = *Direction::all().choose(rng).unwrap();
        if let (Some(Cell::Builder), Some(Cell::Floor)) =
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

    pub fn avg_visited(&self) -> f32 {
        let total = self.state.visited.area();
        let visited: i32 = self.state.visited.iter_cells().map(|&v| i32::from(v)).sum();
        visited as f32 / total as f32
    }

    pub fn encounters(&self) -> i32 {
        self.encounters
    }

    pub fn max_depth_reached(&self) -> i32 {
        self.encounters
    }

    pub fn relative_wall_edges(&self) -> f32 {
        let cells = &self.state.cells;
        cells.count_edges(|cell| matches!(cell, Cell::Stone)) as f32 / cells.area() as f32
    }

    pub fn print_stats(&self) {
        self.nn.print_stats();
    }
}

impl Simulation for Builders {
    fn step(&mut self) {
        let builder_positions: Vec<_> = self.state.builders.iter().map(|t| t.pos).collect();
        for pos in builder_positions {
            self.kick_dust(pos);
        }
        for t in self.state.builders.iter_mut() {
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
            let builders_nearby: i32 = self
                .state
                .cells
                .neighbours(t.pos)
                .map(|(_, cell)| (cell == Some(Cell::Builder)) as i32)
                .iter()
                .sum();
            let inputs = [
                look(Cell::Floor, Angle::Forward),
                look(Cell::Floor, Angle::Left),
                look(Cell::Floor, Angle::Right),
                look(Cell::Floor, Angle::LeftBack),
                look(Cell::Floor, Angle::RightBack),
                look(Cell::Floor, Angle::Back),
                look(Cell::Builder, Angle::Forward),
                self.state.mass.cell(t.pos).unwrap_or(0).into(),
                builders_nearby as f32,
            ];
            self.encounters += builders_nearby;

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

            if let Some(cell_forward) = self.state.cells.cell(pos_forward) {
                match action {
                    Action::Pullback => {
                        if let Some(cell_back) = self.state.cells.cell(pos_back) {
                            if cell_back == Cell::Floor {
                                if cell_forward == Cell::Stone {
                                    self.state.cells.set_cell(pos_forward, Cell::Floor);
                                    self.state.cells.set_cell(t.pos, cell_forward);
                                } else {
                                    self.state.cells.set_cell(t.pos, Cell::Floor);
                                }
                                t.pos = pos_back;
                                self.state.cells.set_cell(t.pos, Cell::Builder);
                            }
                        }
                    }
                    Action::Forward => {
                        match cell_forward {
                            Cell::Stone => {
                                // push
                                let pos_forward2x = pos_forward + t.heading;
                                if let Some(cell_forward2x) = self.state.cells.cell(pos_forward2x) {
                                    if cell_forward2x == Cell::Floor {
                                        self.state.cells.set_cell(pos_forward2x, cell_forward);
                                        self.state.cells.set_cell(t.pos, Cell::Floor);
                                        t.pos = pos_forward;
                                        self.state.cells.set_cell(t.pos, Cell::Builder);
                                    }
                                }
                            }
                            Cell::Floor => {
                                self.state.cells.set_cell(t.pos, Cell::Floor);
                                t.pos = pos_forward;
                                self.state.cells.set_cell(t.pos, Cell::Builder);
                            }
                            _ => {}
                        }
                    }
                    Action::Left | Action::Right => {
                        if cell_forward == Cell::Floor {
                            self.state.cells.set_cell(t.pos, Cell::Floor);
                            t.pos = pos_forward;
                            self.state.cells.set_cell(t.pos, Cell::Builder);
                        }
                    }
                }
            }
            self.state.visited.set_cell(t.pos, true);
            let center: coords::Cube = CENTER.into();
            self.max_depth_reached = self.max_depth_reached.max(center.distance(t.pos));
        }
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self.state).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        self.state = bincode::deserialize_from(data).unwrap();
    }
}

impl HexgridView for Builders {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
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

        let energy: u8 = self.state.mass.cell(pos)?;
        let cell_type = match self.state.cells.cell(pos)? {
            Cell::Floor => 2,
            Cell::Stone => 4,
            Cell::Border => 255,
            Cell::Builder => 0,
        };
        Some(CellView {
            cell_type,
            energy: Some(energy),
            ..Default::default()
        })
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        coords::Rectangle {
            pos: coords::Cube { x: 6, y: -7 }, // should calculate...
            width: RING_RADIUS * 2 - 1,
            height: RING_RADIUS * 2 - 1,
        }
    }
}
