#[derive(PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
enum Cell {
    Border,
    Air,
    Wall,
    Seed,
    Worker,
    Object1,
    Object2,
}
use std::fmt::Debug;

use rand::{thread_rng, Rng, SeedableRng};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use Cell::*;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
struct Hex {
    cell: Cell,
    humidity: u8,
    vapour: BitParticles,
}

// impl Cell {
//     fn max_humidity(&self) -> u8 {
//         match &self {
//             Border | Wall => 0,
//             _ => 64,
//         }
//     }
// }

const BORDER: Hex = Hex {
    cell: Border,
    humidity: 0,
    vapour: BitParticles::EMPTY,
};

use crate::{
    coords, tiled::load_axial_tile_from_json, AxialTile, BitParticles, CellView, Direction,
    DirectionSet, HexgridView, SimRng, Simulation,
};

#[derive(Serialize, Deserialize)]
pub struct HiveSim {
    hexes: AxialTile<Hex>,
    rng: SimRng,
}

pub fn new() -> HiveSim {
    static JSON: &str = include_str!("../../../../maps/testmap.tmj");
    let map = load_axial_tile_from_json(JSON, Border, |idx| match idx {
        0 => Border,
        2 => Wall,
        6 => Air,
        10 => Seed,
        13 => Object1,
        16 => Object2,
        _ => Border,
    });
    let mut rng = Pcg32::from_rng(thread_rng()).unwrap();
    let mut vapour_init = BitParticles::EMPTY;
    // vapour_init.set_resting(2);
    vapour_init.set_outgoing(DirectionSet::single(Direction::West));
    let map = map.map(|cell| Hex {
        cell,
        humidity: match cell {
            Object1 => rng.gen_range(0..=255),
            _ => 0,
        },
        vapour: if matches!(cell, Air) {
            vapour_init
        } else {
            BitParticles::EMPTY
        },
    });

    HiveSim { hexes: map, rng }
}

impl Simulation for HiveSim {
    fn step(&mut self) {
        self.hexes = self.hexes.ca_step(BORDER, |nh| {
            let center = nh.center;
            let mut hex = center;

            hex.vapour = BitParticles::ca_step(nh.map(|h| h.vapour));
            match hex.cell {
                Border | Wall => hex.vapour.reflect_all(),
                Object1 => {
                    hex.vapour.set_outgoing(DirectionSet::none());
                }
                _ => {
                    // sometimes leak resting vapour particles
                    // if hex.vapour.resting() == 2 {
                    //     if self.rng.gen::<u8>() < 100 {
                    //         hex.vapour.set_resting(1);
                    //     }
                    // }
                    // if hex.vapour.resting() == 0 {
                    // if self.rng.gen::<u8>() < 10 {
                    //     hex.vapour.set_resting(0);
                    // }
                    // }
                    if self.rng.gen::<u8>() < 200 {
                        hex.vapour.set_resting(1);
                    } else {
                        if hex.vapour.resting() == 1 {
                            hex.vapour.set_resting(0);
                        }
                    }

                    if self.rng.gen::<u8>() < 3 {
                        hex.vapour.shuffle8_cheap(&mut self.rng);
                        // hex.vapour.swap_random_neighbours(&mut self.rng);
                    }

                    // random walk (more or less)
                    // hex.vapour.shuffle8_cheap_4x(&mut self.rng);

                    // if self.rng.gen::<u8>() < 10 {
                    //     hex.vapour.swap_random_neighbours(&mut self.rng);
                    // }

                    use Direction::*;

                    // strong downwards bias
                    let mut o = hex.vapour.outgoing();
                    o = o.swapped(SouthWest, SouthEast); // straight down (zig-zag)
                    o = o.swapped(NorthWest, NorthEast); // straight up

                    if self.rng.gen::<u8>() < 250 {
                        if o.contains(NorthEast) {
                            o = o.swapped(NorthEast, East)
                        } else if !o.contains(SouthEast) {
                            o = o.swapped(SouthEast, East)
                        }
                        if o.contains(NorthWest) {
                            o = o.swapped(NorthWest, West)
                        } else if !o.contains(SouthWest) {
                            o = o.swapped(SouthWest, West)
                        }
                    }

                    hex.vapour.set_outgoing(o);
                }
            }

            hex
            /*
            let mut transfer_total = 0i32;
            for (dir, neigh) in nh.iter_dirs() {
                use Direction::*;
                fn flow(dir: Direction) -> i32 {
                    match dir {
                        NorthWest => 2,
                        NorthEast => 2,
                        West => 2,
                        East => 2,
                        SouthWest => 4,
                        SouthEast => 4,
                    }
                }
                let maximum_total_transfer = 16;
                let mut transfer = 0i32;
                // transfer += center.humidity as i32 * flow(dir);
                // transfer -= neigh.humidity as i32 * flow(-dir);
                // transfer = transfer.clamp(-1, 1);
                transfer += flow(dir);
                transfer -= flow(-dir);
                if transfer > 0 {
                    if center.humidity < maximum_total_transfer {
                        transfer = 0;
                    }
                }
                if transfer < 0 {
                    if neigh.humidity < maximum_total_transfer {
                        transfer = 0;
                    }
                }
                transfer_total += transfer;
            }
            let mut hex = center;
            // let avg = nh.neighbours.iter().map(|n| n.humidity as u16).sum::<u16>() / 6;

            // assert_eq!((hex.humidity + transfer_total) as u8 as i32, hex.humidity as i32 + transfer_total);
            hex.humidity = (hex.humidity as i32 - transfer_total - 1).clamp(0, 255) as u8;
             */
        });
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

impl HexgridView for HiveSim {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        let hex = self.hexes.cell(pos)?;
        let cell_type = match hex.cell {
            Border => return None,
            Air => 2,
            Wall => 4,
            Seed => 5,
            Worker => 0,
            Object1 => 3,
            Object2 => 1,
        };
        Some(CellView {
            cell_type,
            energy: Some(hex.vapour.outgoing().count() * 2),
            // energy: Some(hex.humidity),
            ..Default::default()
        })
    }

    fn cell_text(&self, pos: coords::Cube) -> Option<String> {
        let cell = self.hexes.cell(pos)?;
        Some(format!("{cell:?}"))
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        self.hexes.viewport()
    }
}
