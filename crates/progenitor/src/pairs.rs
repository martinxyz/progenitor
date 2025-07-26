use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::AxialTile;
use crate::CellView;
use crate::Direction;
use crate::HexgridView;
use crate::SimRng;
use crate::Simulation;
use crate::coords;
use crate::hexmap;

const RADIUS: i32 = 27;

#[derive(Serialize, Deserialize)]
pub struct World {
    cells: AxialTile<Option<Resolution>>,
    rng: SimRng,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
enum Resolution {
    Frozen,
    Source(Direction),
    Target(Direction),
}
use Resolution::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Params {
    pub p0: u8,
    pub p1: u8,
    pub p2: u8,
    pub count0: u8,
    pub count1: u8,
}

impl World {
    pub fn new(params: Params) -> World {
        let mut rng = SimRng::from_rng(&mut rand::rng());

        let init = hexmap::new(RADIUS, None, |_location| Some(rng.random::<u8>()));
        let init = init.ca_step(None, |location| {
            let center: u8 = location.center?;
            // let neighbours_sum1: i16 = location
            //     .neighbours_pairs()
            //     .map(|(a, b)| (a.unwrap_or(128) as i16 - b.unwrap_or(128) as i16).abs())
            //     .sum();
            //     .count() as u8;

            let neighbours_sum2: i16 = location
                .neighbours
                .map(|v| v.unwrap_or(128) as i16)
                .into_iter()
                .sum::<i16>();

            if neighbours_sum2 - center as i16 * 6 < params.p0 as i16 * 4 {
                Some(250)
            } else {
                Some(center)
            }
        });
        let init = init.ca_step(None, |location| {
            // ... also, when using u8 (no src/targt distinction) it was ~2x faster
            // (... may be easier to generate only sources, then create matching sinks where unambiguous)
            Some({
                let neighbours_sum: u16 = location
                    .neighbours
                    .map(|v| v.unwrap_or(128) as u16)
                    .into_iter()
                    .sum();
                let dice: u8 = rng.random_range(0..=(8 + 3));
                match dice {
                    0..=6 => {
                        let dir = Direction::from_int(dice.into());
                        if (neighbours_sum as i16 - (location.center? as i16 * 6))
                            > (params.p1 as i16 - 128) * 8
                        {
                            if (neighbours_sum as i16
                                - (location.neighbour(dir).unwrap_or(128) as i16 * 6) / 8)
                                < (params.p2 as i16 + 1000)
                            {
                                Source(dir)
                            } else {
                                Target(dir)
                            }
                        } else {
                            Frozen
                        }
                    }
                    _ => Frozen,
                }
            })
        });

        World { cells: init, rng }
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = SimRng::seed_from_u64(seed);
    }
}

impl Simulation for World {
    fn step(&mut self) {
        // freeze everything with an invalid resolution
        self.cells = self.cells.ca_step(None, |location| {
            let center = location.center?;
            for (dir, neigh) in location.iter_dirs() {
                let Some(neigh) = neigh else {
                    return Some(Frozen);
                };
                // I think this is pretty slow...
                let valid = match (center, neigh) {
                    (Frozen, _) => true,
                    (Source(dir1), Target(dir2)) => dir1 != dir || dir2 == -dir1,
                    (Target(dir1), Source(dir2)) => dir1 != dir || dir2 == -dir1,
                    (Source(dir1) | Target(dir1), Frozen) => dir1 != dir,
                    _ => false,
                };
                if !valid {
                    return Some(Frozen);
                }
            }
            location.center
        })
    }

    fn save_state(&self) -> Vec<u8> {
        // can we have a default-implementation for Simulation: Serialize + Deserialize
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

pub fn random_params(rng: &mut impl Rng) -> Params {
    Params {
        p0: rng.random(),
        p1: rng.random(),
        p2: rng.random(),
        count0: rng.random_range(0..=6),
        count1: rng.random_range(0..=6),
    }
}

pub fn run_and_evaluate(params: Params) -> f32 {
    let mut world = World::new(params);
    world.steps(3); // 2 should be enough?
    let active = world
        .cells
        .iter_cells()
        .map(|&state| match state {
            Some(Source(_) | Target(_)) => 1usize,
            _ => 0usize,
        })
        .sum::<usize>();
    let total = world.cells.area();
    active as f32 / total as f32
}

impl HexgridView for World {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        self.cells.cell(pos).map(|state| CellView {
            cell_type: match state {
                Some(Frozen) => 0,
                Some(Source(_)) => 1,
                Some(Target(_)) => 2,
                None => 255,
            },
            direction: match state {
                Some(Source(dir)) | Some(Target(dir)) => Some(dir),
                _ => None,
            },
            ..Default::default()
        })
    }
    fn viewport_hint(&self) -> coords::Rectangle {
        hexmap::viewport(RADIUS)
    }
}
