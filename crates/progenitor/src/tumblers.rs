use hex2d::Direction;
use rand::distributions::Bernoulli;
use rand::distributions::Distribution;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::AxialTile;
use crate::CellView;
use crate::HexgridView;
use crate::Simulation;

// Random walk test.

#[derive(Serialize, Deserialize)]
struct Tumbler {
    pos: coords::Cube,
    heading: Direction,
}

#[derive(Serialize, Deserialize)]
pub struct Tumblers {
    visited: AxialTile<bool>,
    tumblers: Vec<Tumbler>,
    rng: rand_pcg::Lcg64Xsh32,
    tumble_prob: f64,
}

const TILE_WIDTH: i32 = 18;
const TILE_HEIGHT: i32 = 29;

impl Tumblers {
    pub fn new(tumble_prob: f64) -> Tumblers {
        let seed = thread_rng().next_u64();
        let mut rng: rand_pcg::Lcg64Xsh32 = Pcg32::seed_from_u64(seed);
        let center = coords::Offset {
            // would be easier in axial coordinates...
            col: (2 * TILE_WIDTH + TILE_HEIGHT) / 4,
            row: TILE_HEIGHT / 2,
        };
        let create_tumbler = |_| Tumbler {
            pos: center.into(),
            heading: *Direction::all().choose(&mut rng).unwrap(),
        };
        Tumblers {
            visited: AxialTile::new(TILE_WIDTH, TILE_HEIGHT, false),
            tumblers: (0..32).map(create_tumbler).collect(),
            rng,
            tumble_prob,
        }
    }
    pub fn avg_visited(&self) -> f32 {
        let total = self.visited.area();
        let visited: i32 = self.visited.iter_cells().map(|&v| i32::from(v)).sum();
        visited as f32 / total as f32
    }
}

impl Simulation for Tumblers {
    fn step(&mut self) {
        let tumble_dist = Bernoulli::new(self.tumble_prob).unwrap();
        for t in self.tumblers.iter_mut() {
            if tumble_dist.sample(&mut self.rng) {
                t.heading = *Direction::all().choose(&mut self.rng).unwrap();
            }
            let new_pos = t.pos + t.heading;
            if self.visited.valid(new_pos) {
                t.pos = new_pos;
                self.visited.set_cell(t.pos, true);
            }
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
        // xxx inefficient when this gets called for all cells...
        for t in self.tumblers.iter() {
            if pos == t.pos {
                return Some(CellView {
                    cell_type: 0,
                    direction: Some(t.heading),
                    ..Default::default()
                });
            }
        }
        Some(CellView {
            cell_type: match self.visited.cell(pos)? {
                false => 1,
                true => 2,
            },
            ..Default::default()
        })
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        self.visited.viewport()
    }
}
