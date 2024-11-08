use std::array;

use rand::thread_rng;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::hexmap;
use crate::AxialTile;
use crate::CellView;
use crate::HexgridView;
use crate::SimRng;
use crate::Simulation;

const RADIUS: i32 = 17;
const MAX_CELL_TYPES: u8 = 6;
const GROWTH_REQURIEMENT: u16 = 12;
const INITIAL_ENERGY: u16 = 2000;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Cell {
    rule: u8,
    energy: u16,
}

impl Cell {
    const EMPTY: Cell = Cell { rule: 0, energy: 0 };
}

#[derive(Serialize, Deserialize)]
pub struct GrowthSim {
    rng: SimRng,
    rules: [CellType; MAX_CELL_TYPES as usize],
    state: AxialTile<Option<Cell>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CellType {
    flow: [u8; 6], // 0 = full stop flow to/from this direction
    growth: [u8; 6],
}

impl CellType {
    fn new_inert() -> Self {
        Self {
            flow: [0; 6],
            growth: [0; 6],
        }
    }
    fn new_random(rng: &mut impl Rng) -> Self {
        Self {
            flow: array::from_fn(|_| rng.gen_range(0..=4)),
            growth: array::from_fn(|_| rng.gen_range(0..MAX_CELL_TYPES)),
        }
    }
}

impl GrowthSim {
    pub fn new() -> Self {
        let seed = thread_rng().next_u64();
        Self::new_with_seed(seed)
    }

    pub fn new_with_seed(seed: u64) -> Self {
        let mut rng = SimRng::seed_from_u64(seed);
        let state = hexmap::new(RADIUS, None, |location| {
            Some({
                if location.dist_from_center() == 0 {
                    Cell {
                        rule: 1,
                        energy: INITIAL_ENERGY,
                    }
                } else {
                    Cell::EMPTY
                }
            })
        });
        let rules = array::from_fn(|i| {
            if i == 0 {
                CellType::new_inert()
            } else {
                CellType::new_random(&mut rng)
            }
        });
        Self { rng, rules, state }
    }
}

impl Simulation for GrowthSim {
    fn step(&mut self) {
        self.state = self.state.ca_step(None, |neighbourhood| {
            let Some(neighbourhood) = neighbourhood.valid_only() else {
                return neighbourhood.center;
            };
            if neighbourhood.center.rule == 0 {
                // no energy flow, but a neighbour may grow a cell here
                let grow_into = neighbourhood
                    .iter_dirs()
                    .map(|(dir, neigh)| {
                        if neigh.energy >= GROWTH_REQURIEMENT {
                            self.rules[neigh.rule as usize].growth[(-dir) as usize]
                        } else {
                            0
                        }
                    })
                    .max()
                    .unwrap();
                Some(Cell {
                    rule: grow_into,
                    energy: 0,
                })
            } else {
                // only energy flow, no transformations
                //
                // Energy transfer happens between two cells. It is based only
                // on information that both cells can see. To keep things
                // simple, we allow a cell to transfer energy away only if it
                // could transfer the same amount to all 6 neighbours, otherwise
                // it could transfer more than it has.
                //
                let mut energy_transfer: i32 = 0;
                for (dir, neigh) in neighbourhood.iter_dirs() {
                    let flow1 = self.rules[neighbourhood.center.rule as usize].flow[dir as usize];
                    let flow2 = self.rules[neigh.rule as usize].flow[-dir as usize];
                    let flow = u8::min(flow1, flow2);

                    let energy1 = neighbourhood.center.energy;
                    let energy2 = neigh.energy;

                    if energy1 > energy2 {
                        if energy1 >= flow as u16 * 6 {
                            energy_transfer -= flow as i32;
                        }
                    } else if energy2 > energy1 {
                        if energy2 >= flow as u16 * 6 {
                            energy_transfer += flow as i32;
                        }
                    }
                }
                let mut energy: i32 = neighbourhood.center.energy.into();
                energy += energy_transfer;
                assert!(energy >= 0);
                let energy = energy.clamp(0, 0xFFFF) as u16;
                Some(Cell {
                    rule: neighbourhood.center.rule,
                    energy,
                })
            }
        });
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

impl HexgridView for GrowthSim {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        let cell: Cell = self.state.cell(pos)??;
        Some(CellView {
            cell_type: cell.rule,
            direction: None,
            energy: Some(cell.energy.clamp(0, 255) as u8),
        })
    }
    fn cell_text(&self, pos: coords::Cube) -> Option<String> {
        let cell = self.state.cell(pos)??;
        Some(format!("{cell:?}"))
    }
    fn viewport_hint(&self) -> coords::Rectangle {
        hexmap::viewport(RADIUS)
    }
}
