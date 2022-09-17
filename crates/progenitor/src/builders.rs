use hex2d::Angle;
use hex2d::Direction;
use hex2d::Spin;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::AxialTile;
use crate::CellView;
use crate::Simulation;

// Random walk test.

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
            _ => panic!(),
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
pub struct Builders {
    cells: AxialTile<Cell>,
    visited: AxialTile<bool>,
    builders: Vec<Builder>,
    rng: rand_pcg::Lcg64Xsh32,
}

const TILE_WIDTH: i32 = 40;
const TILE_HEIGHT: i32 = 40;

impl Builders {
    pub fn new() -> Builders {
        let seed = thread_rng().next_u64();
        let mut rng: rand_pcg::Lcg64Xsh32 = Pcg32::seed_from_u64(seed);
        let center = coords::Offset {
            // would be easier in axial coordinates...
            col: (2 * TILE_WIDTH + TILE_HEIGHT) / 4,
            row: TILE_HEIGHT / 2,
        };
        let create_builder = |_| Builder {
            pos: center.into(),
            heading: *Direction::all().choose(&mut rng).unwrap(),
        };
        let mut cells = AxialTile::new(TILE_WIDTH, TILE_HEIGHT, Cell::Border);

        let center: coords::Cube = center.into();

        for radius in 0..18 {
            for pos in center.ring_iter(radius, Spin::CCW(Direction::XY)) {
                cells.set_cell(
                    pos,
                    match radius {
                        0..=4 => Cell::Air,
                        _ => Cell::Stone,
                    },
                );
            }
        }
        Builders {
            visited: AxialTile::new(TILE_WIDTH, TILE_HEIGHT, false),
            cells,
            builders: (0..5).map(create_builder).collect(),
            rng,
        }
    }
    pub fn avg_visited(&self) -> f32 {
        let total = self.visited.area();
        let visited: i32 = self.visited.iter_cells().map(|&v| i32::from(v)).sum();
        visited as f32 / total as f32
    }
}

impl Default for Builders {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulation for Builders {
    fn step(&mut self) {
        let action_distr = WeightedIndex::new([
            4, // Forward
            1, // Left
            1, // Right
            2, // Pullback
        ])
        .unwrap();
        for t in self.builders.iter_mut() {
            let action = action_distr.sample(&mut self.rng).into();
            let turn = match action {
                Action::Left => Angle::Left,
                Action::Right => Angle::Right,
                _ => Angle::Forward,
            };
            t.heading = t.heading + turn;

            let pos_forward = t.pos + t.heading;
            let pos_back = t.pos - t.heading;

            if let Some(cell_forward) = self.cells.get_cell(pos_forward) {
                if action != Action::Pullback {
                    if cell_forward == Cell::Air {
                        t.pos = pos_forward;
                    }
                } else if let Some(cell_back) = self.cells.get_cell(pos_back) {
                    if cell_back == Cell::Air {
                        let cell_here = self.cells.get_cell(t.pos).unwrap();
                        self.cells.set_cell(pos_back, cell_here);
                        self.cells.set_cell(t.pos, cell_forward);
                        self.cells.set_cell(pos_forward, Cell::Air);
                        t.pos = pos_back;
                    }
                }
            }
            self.visited.set_cell(t.pos, true);
        }
    }

    fn get_cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        let pos = coords::Cube {
            // hack to translate web UI the viewport a bit
            x: pos.x + 12,
            y: pos.y - 12,
        };
        // xxx inefficient when this gets called for all cells...
        for t in self.builders.iter() {
            if pos == t.pos {
                return Some(CellView {
                    cell_type: 0,
                    direction: Some(t.heading),
                    ..Default::default()
                });
            }
        }

        let energy: u8 = self.visited.get_cell(pos)?.into();
        let cell_type = match self.cells.get_cell(pos)? {
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
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}