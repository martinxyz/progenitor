use rand::thread_rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;

use crate::coords;
use crate::tile;
use crate::tile::Tile;
use crate::CellView;
use crate::Simulation;

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

impl Simulation<bool> for World {
    fn tick(&mut self) {
        self.alive = self
            .alive
            .iter_radius_1()
            .map(|(_center, neighbours)| neighbours.iter().any(|&(_, alive)| alive))
            .collect();
    }

    fn get_cells_rectangle(&self) -> Vec<bool> {
        let pos = coords::Cube { x: 0, y: 0 };
        tile::iterate_rectangle(pos, tile::SIZE as i32, tile::SIZE as i32)
            .map(|coord| self.alive.get_cell(coord))
            .collect()
    }
}

impl CellView for bool {
    fn cell_type(&self) -> u8 {
        match self {
            false => 0,
            true => 1,
        }
    }

    fn energy(&self) -> Option<u8> {
        None
    }

    fn direction(&self) -> Option<hex2d::Direction> {
        None
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
