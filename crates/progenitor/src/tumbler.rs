use hex2d::Direction;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::tile::Tile;
use crate::CellView;
use crate::Simulation;

// Random walk test.

#[derive(Serialize, Deserialize)]
pub struct Tumbler {
    pub visited: Tile<bool>,
    pos: coords::Cube,
    rng: rand_pcg::Lcg64Xsh32,
}

impl Tumbler {
    pub fn new() -> Tumbler {
        let seed = thread_rng().next_u64();
        let rng: rand_pcg::Lcg64Xsh32 = Pcg32::seed_from_u64(seed);
        Tumbler {
            visited: Tile::new(false),
            pos: coords::Cube { x: 15, y: 15 },
            rng,
        }
    }
}

impl Simulation for Tumbler {
    fn step(&mut self) {
        let direction: Direction = *Direction::all().choose(&mut self.rng).unwrap();
        self.pos = self.pos + direction;
        self.visited.set_cell(self.pos, true);
    }

    fn get_cell_view(&self, pos: coords::Cube) -> CellView {
        CellView {
            cell_type: if self.visited.is_same_pos(pos, self.pos) {
                0
            } else {
                match self.visited.get_cell(pos) {
                    false => 1,
                    true => 2,
                }
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
