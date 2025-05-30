use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ca;
use crate::coords;
use crate::coords::Direction;
use crate::CellView;
use crate::HexgridView;
use crate::Neighbourhood;
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
enum Cell {
    Air,
    Sand,
    Dust(Option<Direction>),
    Grass,
}

struct Rule;

impl ca::TransactionalCaRule for Rule {
    type Cell = Cell;

    fn transaction(
        &self,
        source: Cell,
        target: Cell,
        direction: Direction,
    ) -> Option<ca::TransactionResult<Cell>> {
        let swap = Some(ca::TransactionResult {
            source: target,
            target: source,
        });
        match (source, target) {
            (Cell::Sand, Cell::Air) => match direction {
                Direction::SouthEast | Direction::SouthWest => swap,
                _ => None,
            },
            (Cell::Dust(None), Cell::Air) => swap,
            (Cell::Dust(Some(dir)), Cell::Air) => {
                if dir == direction {
                    swap
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn step(&self, nh: Neighbourhood<Cell>, rng: &mut SimRng) -> Cell {
        if let Cell::Dust(_) = nh.center {
            Cell::Dust(match rng.random_range(0..16) {
                0 => Some(Direction::SouthWest),
                1 => Some(Direction::SouthEast),
                _ => None,
            })
        } else {
            nh.center
        }
    }
}

impl World {
    pub fn new() -> World {
        let mut rng = SimRng::from_rng(&mut rand::rng());
        World {
            cells: TorusTile::from_fn(|pos| {
                if pos.z() > SIZE as i32 - 3 {
                    Cell::Grass
                } else if pos.z() < 5 {
                    Cell::Sand
                } else if rng.random_bool(0.01) {
                    Cell::Dust(None)
                } else if rng.random_bool(0.02) {
                    Cell::Sand
                } else if pos.z() > 20 && pos.z() < 24 && rng.random_bool(0.6) {
                    Cell::Grass
                } else {
                    Cell::Air
                }
            }),
            rng,
        }
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = SimRng::seed_from_u64(seed);
    }
}

impl Simulation for World {
    fn step(&mut self) {
        let rule = Rule {};
        self.cells = ca::step_torus(&self.cells, &rule, &mut self.rng);
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
                Cell::Sand => 4,
                Cell::Dust(_) => 5,
            },
            direction: if let Cell::Dust(dir) = cell {
                dir
            } else {
                None
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
