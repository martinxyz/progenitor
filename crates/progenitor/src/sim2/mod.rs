use rand::thread_rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::tile;
use crate::tile::Tile;
use crate::CellView;
use crate::Simulation;

#[derive(Serialize, Deserialize)]
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
}

impl Simulation for World {
    fn step(&mut self) {
        self.alive = self
            .alive
            .iter_radius_1()
            .map(|(_center, neighbours)| neighbours.iter().any(|&(_, alive)| alive))
            .collect();
    }

    fn get_cells_rectangle(&self) -> Vec<CellView> {
        let pos = coords::Cube { x: 0, y: 0 };
        tile::iterate_rectangle(pos, tile::SIZE as i32, tile::SIZE as i32)
            .map(|coord| self.alive.get_cell(coord))
            .map(|alive| CellView {
                cell_type: match alive {
                    false => 0,
                    true => 1,
                },
                ..Default::default()
            })
            .collect()
    }

    fn get_cell_view(&self, pos: coords::Cube) -> CellView {
        CellView {
            // FIXME: duplication
            cell_type: match self.alive.get_cell(pos) {
                false => 0,
                true => 1,
            },
            ..Default::default()
        }
    }

    fn save_state(&self) -> Vec<u8> {
        // can we have a default-implementation for Simulation: Serialize + Deserialize
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
