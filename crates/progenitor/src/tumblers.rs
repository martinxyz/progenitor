use rand::distributions::Bernoulli;
use rand::distributions::Distribution;
use rand::prelude::SliceRandom;
use rand::thread_rng;
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

// Random walk test.

#[derive(Serialize, Deserialize)]
struct Tumbler {
    pos: coords::Cube,
    heading: Direction,
}

#[derive(Serialize, Deserialize)]
pub struct Tumblers {
    visited: AxialTile<Option<bool>>,
    tumblers: Vec<Tumbler>,
    rng: SimRng,
    tumble_prob: f64,
}

const RADIUS: i32 = 12;

impl Tumblers {
    pub fn new(tumble_prob: f64) -> Tumblers {
        let seed = thread_rng().next_u64();
        let mut rng = SimRng::seed_from_u64(seed);
        let create_tumbler = |_| Tumbler {
            pos: hexmap::center(RADIUS),
            heading: *Direction::all().choose(&mut rng).unwrap(),
        };
        Tumblers {
            visited: hexmap::new(RADIUS, None, |_location| Some(false)),
            tumblers: (0..32).map(create_tumbler).collect(),
            rng,
            tumble_prob,
        }
    }
    pub fn avg_visited(&self) -> f32 {
        let total = self.visited.area();
        let visited: i32 = self
            .visited
            .iter_cells()
            .map(|&v| match v {
                Some(true) => 1,
                _ => 0,
            })
            .sum();
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
            if matches!(self.visited.cell(new_pos), Some(Some(_))) {
                t.pos = new_pos;
                self.visited.set_cell(t.pos, Some(true));
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
                None => 255,
                Some(false) => 1,
                Some(true) => 2,
            },
            ..Default::default()
        })
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        hexmap::viewport(RADIUS)
    }
}
