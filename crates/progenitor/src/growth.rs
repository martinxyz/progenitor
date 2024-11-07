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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Cell {
    rule: u8,
    energy: u16,
}

impl Cell {
    const EMPTY: Cell = Cell { rule: 0, energy: 0 };
}

const MAX_CELL_TYPES: u8 = 4;

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

const RADIUS: i32 = 12;

impl GrowthSim {
    pub fn new() -> Self {
        let seed = thread_rng().next_u64();
        Self::new_with_seed(seed)
        // Self::new_with_seed(33)
    }

    pub fn new_with_seed(seed: u64) -> Self {
        let mut rng = SimRng::seed_from_u64(seed);
        let state = hexmap::new(RADIUS, None, |location| {
            Some({
                if location.dist_from_center() == 0 {
                    Cell {
                        rule: 1,
                        energy: 1000,
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
        // // let tumble_dist = Bernoulli::new(self.tumble_prob).unwrap();
        // self.state = self.state.ca_step(Cell::BORDER, |neighbourhood| {
        //     match neighbourhood.center.kind {
        //         CellType::Air => self.air_rule.step(neighbourhood, &mut self.rng),
        //         CellType::Blob => self.blob_rule.step(neighbourhood, &mut self.rng),
        //         _ => neighbourhood.center,
        //     }
        // });

        // for (cell, visited) in self.state.iter_cells().zip(self.visited.iter_cells_mut()) {
        //     *visited = visited.map(|visited| visited || cell.kind == CellType::Blob);
        // }
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
