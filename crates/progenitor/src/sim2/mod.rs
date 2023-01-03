use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::CellView;
use crate::HexgridView;
use crate::Simulation;
use crate::TorusTile;
use crate::SIZE;

#[derive(Serialize, Deserialize)]
pub struct World {
    alive: TorusTile<bool>,
    rng: Pcg32,
}

impl World {
    pub fn new() -> World {
        let mut rng = Pcg32::from_rng(thread_rng()).unwrap();
        World {
            alive: (0..SIZE * SIZE)
                .into_iter()
                .map(|_| rng.gen_bool(0.03))
                .collect(),
            rng,
        }
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = Pcg32::seed_from_u64(seed);
    }
}

impl Simulation for World {
    fn step(&mut self) {
        self.alive = self
            .alive
            .iter_radius_1()
            .map(|(_center, neighbours)| neighbours.iter().any(|&(_, alive)| alive))
            .collect();
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
        Some(CellView {
            cell_type: match self.alive.cell(pos) {
                false => 0,
                true => 1,
            },
            ..Default::default()
        })
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
