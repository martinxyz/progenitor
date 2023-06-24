use rand::distributions::Bernoulli;
use rand::distributions::Distribution;
use rand::prelude::SliceRandom;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::coords::Direction;
use crate::hexmap;
use crate::AxialTile;
use crate::CellView;
use crate::HexgridView;
use crate::SimRng;
use crate::Simulation;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum CellType {
    Border,
    Stone,
    Air,
    Blob,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Cell {
    kind: CellType,
    heading: Option<Direction>,
    // source: Option<Direction>,
}

impl Cell {
    const BORDER: Cell = Cell {
        kind: CellType::Border,
        heading: None,
    };
}

#[derive(Serialize, Deserialize)]
pub struct Tumblers {
    state: AxialTile<Cell>,
    visited: AxialTile<Option<bool>>,
    rng: SimRng,
    tumble_prob: f64,
}

const RADIUS: i32 = 12;

impl Tumblers {
    pub fn new(tumble_prob: f64) -> Tumblers {
        let seed = thread_rng().next_u64();
        let mut rng = SimRng::seed_from_u64(seed);
        let mut state = hexmap::new(RADIUS, Cell::BORDER, |location| {
            let random_heading = Direction::try_from(rng.gen::<u8>() as i32 % 8).ok();
            match location.dist_from_center() {
                0..=1 => Cell {
                    kind: CellType::Blob,
                    heading: random_heading,
                },
                2 => Cell {
                    kind: CellType::Air,
                    heading: random_heading,
                },
                _ if rng.gen_bool(0.08) => Cell {
                    kind: CellType::Stone,
                    heading: random_heading,
                },
                _ => Cell {
                    kind: CellType::Air,
                    heading: random_heading,
                },
            }
        });
        state.ca_step(|neighbourhood| {
            if neighbourhood.center.kind == CellType::Air {
                if Direction::all()
                    .iter()
                    .zip(neighbourhood.neighbours.iter())
                    .any(|(&dir, &neigh)| {
                        neigh.kind == CellType::Stone
                            && (neigh.heading == Some(dir) || neigh.heading == Some(-dir))
                    })
                {
                    return Cell {
                        kind: CellType::Stone,
                        heading: None,
                    };
                }
            } else if neighbourhood.center.kind == CellType::Stone {
                return Cell {
                    heading: None,
                    ..neighbourhood.center
                };
            }
            return neighbourhood.center;
        });
        Tumblers {
            state,
            visited: hexmap::new(RADIUS, None, |_location| Some(false)),
            rng,
            tumble_prob,
        }
    }

    pub fn avg_visited(&self) -> f32 {
        let mut visited = 0;
        let mut total = 0;
        for &v in self.visited.iter_cells() {
            if let Some(v) = v {
                total += 1;
                if v {
                    visited += 1;
                }
            }
        }
        visited as f32 / total as f32
    }
}

impl Simulation for Tumblers {
    fn step(&mut self) {
        // let tumble_dist = Bernoulli::new(self.tumble_prob).unwrap();

        for (cell, visited) in self.state.iter_cells().zip(self.visited.iter_cells_mut()) {
            *visited = visited.map(|visited| visited || cell.kind == CellType::Blob);
        }
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

impl HexgridView for Tumblers {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        let cell = self.state.cell(pos).unwrap_or(Cell::BORDER);
        let visited = self.visited.cell(pos).unwrap_or(None);
        Some(CellView {
            cell_type: match cell.kind {
                CellType::Border => 255,
                CellType::Stone => 4,
                CellType::Air => 2,
                CellType::Blob => 0,
            },
            direction: cell.heading,
            energy: visited.map(|v| if v { 1 } else { 0 }),
            ..Default::default()
        })
    }
    fn cell_text(&self, pos: coords::Cube) -> Option<String> {
        let cell = self.state.cell(pos).unwrap_or(Cell::BORDER);
        Some(format!("{cell:?}"))
    }
    fn viewport_hint(&self) -> coords::Rectangle {
        hexmap::viewport(RADIUS)
    }
}
