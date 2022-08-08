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
use crate::tile;
use crate::tile::Tile;
use crate::CellView;
use crate::Simulation;

// Random walk test.

#[derive(Serialize, Deserialize)]
struct Tumbler {
    pos: coords::Cube,
    heading: Direction,
}

#[derive(Serialize, Deserialize)]
pub struct Tumblers {
    visited: Tile<bool>,
    tumblers: Vec<Tumbler>,
    rng: rand_pcg::Lcg64Xsh32,
    tumble_prob: f64,
}

impl Tumblers {
    pub fn new(tumble_prob: f64) -> Tumblers {
        let seed = thread_rng().next_u64();
        let mut rng: rand_pcg::Lcg64Xsh32 = Pcg32::seed_from_u64(seed);
        let size_half = (crate::SIZE / 2) as i32;
        let center = coords::Offset {
            col: size_half,
            row: size_half,
        };
        let create_tumbler = |_| Tumbler {
            pos: center.into(),
            heading: *Direction::all().choose(&mut rng).unwrap(),
        };
        Tumblers {
            visited: Tile::new(false),
            tumblers: (0..32).map(create_tumbler).collect(),
            rng,
            tumble_prob,
        }
    }
    pub fn avg_visited(&self) -> f32 {
        self.visited.iter_cells().filter(|&&v| v).count() as f32
            * (1. / (tile::SIZE * tile::SIZE) as f32)
    }
}

impl Simulation for Tumblers {
    fn step(&mut self) {
        let tumble_dist = Bernoulli::new(self.tumble_prob).unwrap();
        for t in self.tumblers.iter_mut() {
            if tumble_dist.sample(&mut self.rng) {
                t.heading = *Direction::all().choose(&mut self.rng).unwrap();
            }
            t.pos = t.pos + t.heading;
            self.visited.set_cell(t.pos, true);
        }
    }

    fn get_cell_view(&self, pos: coords::Cube) -> CellView {
        // xxx inefficient when this gets called for all cells...
        for t in self.tumblers.iter() {
            if self.visited.is_same_pos(pos, t.pos) {
                return CellView {
                    cell_type: 0,
                    direction: Some(t.heading),
                    ..Default::default()
                };
            }
        }
        CellView {
            cell_type: match self.visited.get_cell(pos) {
                false => 1,
                true => 2,
            },
            ..Default::default()
        }
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}
