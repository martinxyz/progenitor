use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::AxialTile;
use crate::coords;
use crate::CellView;
use crate::HexgridView;
use crate::SimRng;
use crate::Simulation;
use crate::hexmap;

const RADIUS: i32 = 15;

#[derive(Serialize, Deserialize)]
pub struct World {
    cells: AxialTile<Option<bool>>,
    rng: SimRng,
}

impl World {
    pub fn new() -> World {
        let mut rng = Pcg32::from_rng(thread_rng()).unwrap();

        let cells = hexmap::new(RADIUS, None, |_location| Some(rng.gen_bool(0.4)));
        World { cells, rng }
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = Pcg32::seed_from_u64(seed);
    }
}

impl Simulation for World {
    fn step(&mut self) {
        self.cells = self.cells.ca_step(None, |nh| {
            let alive = nh.center?;
            let alive_neighbours = nh.count_neighbours(|n| n == Some(true));
            let good = if alive {
                alive_neighbours <= 1
            } else {
                alive_neighbours >= 1
            };
            if good || self.rng.gen_bool(0.5) {
                Some(alive)
            } else {
                Some(!alive)
            }
        });
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
        let cell = self.cells.cell(pos)?;
        cell.map(|cell| CellView {
            cell_type: match cell {
                false => 0,
                true => 1,
            },
            ..Default::default()
        })
    }
    fn viewport_hint(&self) -> coords::Rectangle {
        hexmap::viewport(RADIUS)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
