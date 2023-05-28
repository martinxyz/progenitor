use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::CellView;
use crate::HexgridView;
use crate::SimRng;
use crate::Simulation;
use crate::TorusTile;
use crate::{SIZE, VIEWPORT};

#[derive(Serialize, Deserialize)]
pub struct World {
    sand: TorusTile<Cell>,
    rng: SimRng,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum Cell {
    Air,
    Sand,
    Grass,
}

impl World {
    pub fn new() -> World {
        let mut rng = Pcg32::from_rng(thread_rng()).unwrap();
        World {
            sand: TorusTile::from_fn(|pos| {
                if pos.z() > SIZE as i32 - 3 {
                    Cell::Grass
                } else if rng.gen_bool(0.12) {
                    Cell::Sand
                } else {
                    Cell::Air
                }
            }),
            rng,
        }
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = Pcg32::seed_from_u64(seed);
    }

    fn move_sand(&mut self, pos: coords::Cube) {
        let dir = match self.rng.gen() {
            false => coords::Direction::ZY,
            true => coords::Direction::ZX,
        };

        let dst = pos + dir;
        match self.sand.cell(dst) {
            Cell::Air => {
                self.sand.set_cell(pos, Cell::Air);
                self.sand.set_cell(dst, Cell::Sand);
            }
            _ => {}
        }
    }
}

impl Simulation for World {
    fn step(&mut self) {
        for _ in 0..((SIZE * SIZE) / 2) as i32 {
            let pos = self.sand.random_pos(&mut self.rng);
            match self.sand.cell(pos) {
                Cell::Sand => self.move_sand(pos),
                _ => {}
            }
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

impl HexgridView for World {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        Some(CellView {
            cell_type: match self.sand.cell(pos) {
                Cell::Air => 0,
                Cell::Grass => 1,
                Cell::Sand => 3,
            },
            ..Default::default()
        })
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        VIEWPORT
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
