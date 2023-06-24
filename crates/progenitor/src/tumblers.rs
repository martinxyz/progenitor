use std::array;

use nalgebra::SVector;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

use crate::axial_tile::Neighbourhood;
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
    air_rule: AirRule,
}

const NEIGHBOURHOOD_FEATURE_LENGTH: usize = 4;

#[derive(Clone, Serialize, Deserialize)]
pub struct AirRule {
    neighbourhood_features: [SVector<i8, NEIGHBOURHOOD_FEATURE_LENGTH>; 2 * 2 * 3],
    neighbourhood_features_thresholds: [i8; NEIGHBOURHOOD_FEATURE_LENGTH],
    transform: [bool; 1 << NEIGHBOURHOOD_FEATURE_LENGTH],
    change_heading: [bool; 1 << NEIGHBOURHOOD_FEATURE_LENGTH],
}

impl AirRule {
    fn new_random(rng: &mut impl Rng) -> Self {
        Self {
            neighbourhood_features: array::from_fn(|_| {
                SVector::from_fn(|_, _| *[-13, -2, -1, 0, 0, 1, 2, 13].iter().choose(rng).unwrap())
            }),
            neighbourhood_features_thresholds: array::from_fn(|_| rng.gen_range(-60..=60)),
            transform: array::from_fn(|_| rng.gen_bool(0.2)),
            change_heading: array::from_fn(|_| rng.gen_bool(0.2)),
        }
    }

    fn step(&self, neighbourhood: Neighbourhood<Cell>, rng: &mut impl Rng) -> Cell {
        let mut features_sum: SVector<i8, NEIGHBOURHOOD_FEATURE_LENGTH> = SVector::zeros();

        let center = neighbourhood.center;
        #[allow(clippy::identity_op)]
        for i in 0..6usize {
            let dir = Direction::from_int(i as i32);
            let neigh = neighbourhood.neighbours[i];
            // let heading_result = match (neigh.center.heading, neigh.

            let center_heads_to_neigh = center.heading.map(|dir2| dir2 == dir).unwrap_or(false);
            let neigh_heads_to_center = neigh.heading.map(|dir2| dir2 == -dir).unwrap_or(false);
            let mut idx = 0usize;
            idx += 1 * neigh_heads_to_center as usize;
            idx += 2 * center_heads_to_neigh as usize;
            idx += 4 * match neigh.kind {
                CellType::Air => 0,
                CellType::Blob => 1,
                CellType::Stone | CellType::Border => 2,
            };
            let feature = self.neighbourhood_features[idx];
            features_sum += feature;
            // features_max[i] = features_max[i].max(feature);
        }
        let mut idx = 0usize;
        let mut pow = 1usize;
        for (sum, threshold) in features_sum
            .iter()
            .zip(self.neighbourhood_features_thresholds)
        {
            if sum > &threshold {
                idx += pow;
            }
            pow *= 2;
        }
        let mut cell = center;
        if self.change_heading[idx] {
            cell.heading = Direction::try_from((rng.gen::<u8>() % 8) as i32).ok();
        }
        if self.transform[idx] {
            cell.kind = CellType::Blob;
        }
        cell
    }
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
            neighbourhood.center
        });
        let air_rule = AirRule::new_random(&mut rng);
        Tumblers {
            state,
            visited: hexmap::new(RADIUS, None, |_location| Some(false)),
            rng,
            tumble_prob,
            air_rule,
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
        self.state
            .ca_step(|neighbourhood| match neighbourhood.center.kind {
                CellType::Air => self.air_rule.step(neighbourhood, &mut self.rng),
                _ => neighbourhood.center,
            });

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
