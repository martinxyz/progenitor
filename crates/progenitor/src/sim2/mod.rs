use hex2d::Direction;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::CellView;
use crate::HexgridView;
use crate::SimRng;
use crate::Simulation;
use crate::TorusTile;
use crate::{SIZE, VIEWPORT};

#[derive(Serialize, Deserialize)]
pub struct World {
    sand: TorusTile<Cell>,
    rng: SimRng,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Cell {
    Air,
    Sand,
    Grass,
}

impl World {
    pub fn new() -> World {
        let mut rng = Pcg32::from_rng(thread_rng()).unwrap();
        World {
            sand: TorusTile::from_fn(|pos| {
                if pos.z() > SIZE as i32 - 3 {
                    Cell::Grass
                } else if rng.gen_bool(0.12) {
                    Cell::Sand
                } else {
                    Cell::Air
                }
            }),
            rng,
        }
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = Pcg32::seed_from_u64(seed);
    }

    fn step_ca_sand(&mut self) {
        let transactions: TorusTile<(Cell, Direction)> = self
            .sand
            .iter_cells()
            .map(|&cell| (cell, *Direction::all().choose(&mut self.rng).unwrap()))
            .collect();
        self.sand = transactions
            .iter_radius_1()
            .map(|(center, neighbours)| {
                let dir = center.1;
                let (_, (neigh, dir2)) = neighbours[center.1 as usize];
                let mut should_swap = false;
                if dir == -dir2 {
                    // "should_swap" must be symmetric (the other cell must reach the same conclusion)
                    // So we just check both ways (...can we make a helper function or something?)
                    for (cell0, dir, cell1) in [(center.0, dir, neigh), (neigh, -dir, center.0)] {
                        if cell0 == Cell::Sand && cell1 == Cell::Air {
                            if dir == Direction::ZY {
                                should_swap = true;
                            }
                        }
                    }
                }
                if should_swap {
                    neigh
                } else {
                    center.0
                }
            })
            .collect();
    }
}

impl Simulation for World {
    fn step(&mut self) {
        self.step_ca_sand()
    }

    fn save_state(&self) -> Vec<u8> {
        // can we have a default-implementation for Simulation: Serialize + Deserialize
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

impl HexgridView for World {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        Some(CellView {
            cell_type: match self.sand.cell(pos) {
                Cell::Air => 0,
                Cell::Grass => 1,
                Cell::Sand => 3,
            },
            ..Default::default()
        })
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        VIEWPORT
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
