use crate::{AxialTile, BitParticles, CellView, HexgridView, SimRng, Simulation, coords};
use crate::{Direction, DirectionSet, hexmap};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use Direction::*;

#[derive(Serialize, Deserialize)]
pub struct World {
    hexes: AxialTile<Hex>,
    rng: SimRng,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
struct Hex {
    cell: Cell,
    mass: BitParticles,
}

// #[derive(Clone, Copy)]
// struct Sextant {
//     mass: bool
// }

// impl Hex {
//     fn sextants(&self) -> [Sextant; 6] {
//         self.mass.outgoing()
//     }
// }

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
enum Cell {
    Border,
    Wall,
    Air,
    Sand,
    Dust,
}
use Cell::*;

impl Cell {
    fn is_obstacle(self) -> bool {
        match self {
            Border | Wall | Sand | Dust => true,
            Air => false,
        }
    }
}

const BORDER: Hex = Hex {
    cell: Border,
    mass: BitParticles::TWO_RESTING,
};

impl Hex {
    fn grow_prio(self) -> u8 {
        self.mass.count()
            * match self.cell {
                Border => 0,
                Wall => 0,
                Air => 0,
                Sand => 1,
                Dust => 2,
            }
    }
}

const RING_RADIUS: i32 = 23;

impl World {
    pub fn new() -> World {
        let mut rng = SimRng::from_rng(&mut rand::rng());
        World {
            hexes: hexmap::new(RING_RADIUS, BORDER, |loc| {
                if loc.dist_from_top() < 4 {
                    Hex {
                        cell: Sand,
                        mass: BitParticles::FULL,
                    }
                } else if loc.dist_from_bottom() < 1 || rng.random_bool(0.05) {
                    Hex {
                        cell: Wall,
                        mass: BitParticles::new(DirectionSet::none(), 1),
                    }
                } else if rng.random_bool(0.01) {
                    Hex {
                        cell: Dust,
                        mass: BitParticles::new(DirectionSet::none(), 1),
                    }
                } else {
                    Hex {
                        cell: Air,
                        mass: BitParticles::EMPTY,
                    }
                }
            }),
            rng,
        }
    }

    // pub fn seed(&mut self, seed: u64) {
    //     self.rng = SimRng::seed_from_u64(seed);
    // }
}

impl Simulation for World {
    fn step(&mut self) {
        self.hexes = self.hexes.ca_step(BORDER, |nh| {
            fn is_mass_transfer_allowed(src: Cell, dst: Cell) -> bool {
                match (src, dst) {
                    (Dust, Dust) => true,
                    (Sand, Sand) => true,
                    _ => false,
                }
            }

            let mut transfer_result = DirectionSet::none();
            let mut mass_received = DirectionSet::none();
            for dir in Direction::iter_all() {
                let center = nh.center;
                let neigh = nh.neighbour(dir);
                let has_outgoing = center.mass.outgoing().has(dir);
                let has_incoming = neigh.mass.outgoing().has(-dir);
                let forbidden1 = has_outgoing && !is_mass_transfer_allowed(center.cell, neigh.cell);
                let forbidden2 = has_incoming && !is_mass_transfer_allowed(neigh.cell, center.cell);
                if forbidden1 || forbidden2 {
                    // no transfer (copy old state)
                    transfer_result.set(dir, has_outgoing);
                    mass_received.set(dir, false);
                } else {
                    // swap outgoing with incoming
                    transfer_result.set(dir, has_incoming);
                    mass_received.set(dir, has_incoming);
                }
            }
            let mut mass_result = BitParticles::new(transfer_result, nh.center.mass.resting());

            let mut cell_result = nh.center.cell;

            if mass_result.count() == 0 {
                // no mass, check if a neighbour may grow here (if not, become Air)
                cell_result = nh
                    .iter_dirs()
                    .fold((0u16, Air), |(prio, cell), (dir, neigh)| {
                        let neigh_is_pushing_mass: bool = neigh.mass.outgoing().has(-dir);
                        let neigh_prio =
                            /* neigh.grow_prio() as u16 + */ neigh_is_pushing_mass as u16 * 256;
                        if neigh_prio > prio {
                            (neigh_prio, neigh.cell)
                        } else {
                            (prio, cell)
                        }
                    })
                    .1;
            } else {
                match cell_result {
                    Dust => {
                        // move incoming mass straight ahead (if possible)
                        if mass_received.has(West) || mass_received.has(East) {
                            mass_result.swap_directions(West, East);
                        }
                        if mass_received.has(NorthEast) || mass_received.has(SouthWest) {
                            mass_result.swap_directions(NorthEast, SouthWest);
                        }
                        if mass_received.has(NorthWest) || mass_received.has(SouthEast) {
                            mass_result.swap_directions(NorthWest, SouthEast);
                        }

                        if mass_result.resting() > 0 {
                            // get rid of resting mass
                            mass_result.shuffle8_cheap(&mut self.rng);
                        } else if self.rng.random::<u8>() < 8 {
                            // random action
                            mass_result.shuffle8_cheap_4x(&mut self.rng);
                        } else {
                            // check if we are looking towards an obstacle
                            let mut facing_obstacle = false;
                            for (dir, neigh) in nh.iter_dirs() {
                                if mass_result.outgoing().has(dir) {
                                    if neigh.cell.is_obstacle() {
                                        facing_obstacle = true;
                                    }
                                }
                            }
                            // random rotation
                            if self.rng.random::<u8>() < 40 || facing_obstacle {
                                if self.rng.random::<u8>() < 128 {
                                    mass_result.rotate(1)
                                } else {
                                    mass_result.rotate(-1)
                                }
                            }
                        }
                    }
                    Sand => {
                        if mass_received.has(NorthWest) {
                            mass_result.swap_directions(NorthWest, SouthWest);
                        }
                        if mass_received.has(NorthEast) {
                            mass_result.swap_directions(NorthEast, SouthEast);
                        }
                        let blocked1 = mass_result.outgoing().has(SouthEast) && nh[SouthEast].cell.is_obstacle();
                        let blocked2 = mass_result.outgoing().has(SouthWest) && nh[SouthWest].cell.is_obstacle();
                        // let blocked1 = mass_result.outgoing().has(SouthEast) && !matches!(nh[SouthEast].cell, Air | Sand);
                        // let blocked2 = mass_result.outgoing().has(SouthWest) && !matches!(nh[SouthWest].cell, Air | Sand);
                        if  blocked1 || blocked2 {
                            mass_result.swap_directions(SouthEast, SouthWest);
                        }
                        // else if (!blocked1 || !blocked2) {
                        //     mass_result.shuffle8_cheap(&mut self.rng);
                        // }
                        // if  {
                        //     mass_result.swap_random_neighbours(&mut self.rng);
                        // }
                        // mass_result.reflect_all();
                        // mass_result.shuffle8_cheap_4x(&mut self.rng);
                    }
                    _ => {}
                }
            };

            Hex {
                cell: cell_result,
                mass: mass_result,
            }
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

impl HexgridView for World {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        let hex = self.hexes.cell(pos)?;
        let mut energy = match hex.cell {
            Border | Air | Wall => None,
            Sand | Dust => Some(hex.mass.count()),
        };
        if hex.mass.count() == 0 {
            // hex.cell = Air
            energy = Some(20);
        }
        Some(CellView {
            cell_type: match hex.cell {
                Border => return None,
                Air => 0,
                Wall => 1,
                Sand => 4,
                Dust => 5,
            },
            energy,
            direction: None,
            ..Default::default()
        })
    }
    fn cell_text(&self, pos: coords::Cube) -> Option<String> {
        let hex = self.hexes.cell(pos)?;
        Some(format!("{:?}\n{:?}", hex.cell, hex.mass))
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        self.hexes.viewport()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
