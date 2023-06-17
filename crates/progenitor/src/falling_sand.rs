use hex2d::Direction;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::ca;
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
    dust: bool,
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
                } else if rng.gen_bool(0.2) {
                    Cell::Sand(SandState {
                        dust: rng.gen_bool(0.05),
                        coming_from: None,
                        move_attempt: None,
                    })
                } else if rng.gen_bool(0.15) {
                    Cell::Grass
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
}

fn random_down(rng: &mut impl Rng) -> Direction {
    [Direction::ZY, Direction::ZX]
        .into_iter()
        .choose(rng)
        .unwrap()
}

fn random_dust_dir(rng: &mut impl Rng) -> Direction {
    if rng.gen_bool(0.2) {
        random_down(rng)
    } else {
        Direction::all().iter().copied().choose(rng).unwrap()
    }
}

fn rule(center: Cell, neighbours: ca::Neighbours<Cell>, rng: &mut impl Rng) -> Cell {
    match center {
        Cell::Air => {
            if let Some(coming_from) = neighbours
                .iter()
                .filter_map(|(dir2, neigh)| {
                    if let Cell::Sand(state) = neigh {
                        if let Some(dir) = state.move_attempt {
                            if dir == -dir2 {
                                return Some((dir2, state));
                            }
                        }
                    }
                    None
                })
                .choose(rng)
            {
                Cell::Sand(SandState {
                    dust: coming_from.1.dust,
                    coming_from: Some(coming_from.0),
                    move_attempt: Some((if coming_from.1.dust {
                        random_dust_dir
                    } else {
                        random_down
                    })(rng)),
                })
            } else {
                center
            }
        }
        Cell::Sand(state) => {
            let successfully_moved: bool = neighbours.iter().any(|(dir2, neigh)| match neigh {
                Cell::Sand(SandState {
                    coming_from: Some(dir),
                    ..
                }) => dir == -dir2,
                _ => false,
            });
            if successfully_moved {
                Cell::Air
            } else {
                Cell::Sand(SandState {
                    dust: state.dust,
                    coming_from: None,
                    move_attempt: match state.move_attempt {
                        None => Some((if state.dust {
                            random_dust_dir
                        } else {
                            random_down
                        })(rng)),

                        Some(_) => None,
                    },
                })
            }
        }
        Cell::Grass => center,
    }
}

impl Simulation for World {
    fn step(&mut self) {
        self.cells = ca::step(&self.cells, |center, neighbours| {
            rule(center, neighbours, &mut self.rng)
        });
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
                    None => {
                        if state.dust {
                            5
                        } else {
                            4
                        }
                    }
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
