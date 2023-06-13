use hex2d::Direction;
use rand::seq::IteratorRandom;
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
    cells: TorusTile<Cell>,
    rng: SimRng,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
struct SandState {
    coming_from: Option<Direction>,
    move_attempt: Option<Direction>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
enum Cell {
    Air,
    Sand(SandState),
    Grass,
}

impl World {
    pub fn new() -> World {
        let mut rng = Pcg32::from_rng(thread_rng()).unwrap();
        World {
            cells: TorusTile::from_fn(|pos| {
                if pos.z() > SIZE as i32 - 3 {
                    Cell::Grass
                } else if rng.gen_bool(0.12) {
                    Cell::Sand(SandState {
                        coming_from: None,
                        move_attempt: None,
                    })
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

    fn step_ca(&mut self) {
        fn random_down(rng: &mut impl Rng) -> Direction {
            [Direction::ZY, Direction::ZX]
                .into_iter()
                .choose(rng)
                .unwrap()
        }

        self.cells = self
            .cells
            .iter_radius_1()
            .map(|(center, neighbours)| match center {
                Cell::Air => {
                    if let Some(coming_from) = neighbours
                        .into_iter()
                        .filter_map(|(dir2, neigh)| match neigh {
                            Cell::Sand(SandState {
                                move_attempt: Some(dir),
                                ..
                            }) => {
                                if dir == -dir2 {
                                    Some(dir2)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        })
                        .choose(&mut self.rng)
                    {
                        Cell::Sand(SandState {
                            coming_from: Some(coming_from),
                            move_attempt: Some(random_down(&mut self.rng)),
                        })
                    } else {
                        center
                    }
                }
                Cell::Sand(state) => {
                    let successfully_moved: bool =
                        neighbours.iter().any(|(dir2, neigh)| match neigh {
                            Cell::Sand(SandState {
                                coming_from: Some(dir),
                                ..
                            }) => *dir == -*dir2,
                            _ => false,
                        });
                    if successfully_moved {
                        Cell::Air
                    } else {
                        Cell::Sand(SandState {
                            coming_from: None,
                            move_attempt: match state.move_attempt {
                                None => Some(random_down(&mut self.rng)),
                                Some(_) => None,
                            },
                        })
                    }
                }
                Cell::Grass => center,
            })
            .collect();
    }
}

impl Simulation for World {
    fn step(&mut self) {
        self.step_ca()
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
        let cell = self.cells.cell(pos);
        Some(CellView {
            cell_type: match cell {
                Cell::Air => 0,
                Cell::Grass => 1,
                Cell::Sand(state) => match state.coming_from {
                    None => 4,
                    Some(_) => 0,
                },
            },
            direction: match cell {
                Cell::Sand(state) => state.coming_from,
                _ => None,
            },
            ..Default::default()
        })
    }
    fn cell_text(&self, pos: coords::Cube) -> Option<String> {
        let cell = self.cells.cell(pos);
        Some(format!("{cell:?}"))
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
