use rand::thread_rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::tile::Tile;

pub struct World {
    alive: Tile<bool>,
    rng: Pcg32,
}

impl World {
    pub fn new() -> World {
        let rng = Pcg32::from_rng(thread_rng()).unwrap();
        World {
            alive: Tile::new(false),
            rng,
        }
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = Pcg32::seed_from_u64(seed);
    }

    pub fn tick(&mut self) {
        self.alive = self
            .alive
            .iter_radius_1()
            .map(|(_center, neighbours)| neighbours.iter().any(|&(_, alive)| alive))
            .collect();
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
