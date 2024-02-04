use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::hexmap;
use crate::AxialTile;
use crate::CellView;
use crate::HexgridView;
use crate::Neighbourhood;
use crate::SimRng;
use crate::Simulation;

const RADIUS: i32 = 27;

#[derive(Serialize, Deserialize)]
pub struct World {
    frozen: AxialTile<Option<bool>>,
    rng: SimRng,
    phase: u8,
    p0: u8,
}

impl World {
    pub fn new() -> World {
        let rng = Pcg32::from_rng(thread_rng()).unwrap();

        let frozen = hexmap::new(RADIUS, None, |_location| Some(true));
        World {
            frozen,
            rng,
            phase: 0,
            p0: 204, // optimum
        }
    }

    pub fn seed(&mut self, seed: u64) {
        self.rng = Pcg32::seed_from_u64(seed);
    }
}

impl Simulation for World {
    fn step(&mut self) {
        let rng = &mut self.rng;
        self.frozen = match self.phase {
            0 => hexmap::new(RADIUS, None, |_location| Some(rng.gen::<u8>() < self.p0)),
            _ => self.frozen.ca_step(None, remove_invalid),
        };
        self.phase += 1;
        if self.phase > 5 {
            self.randomize_params();
            self.phase = 0;
        }

        fn remove_invalid(nh: Neighbourhood<Option<bool>>) -> Option<bool> {
            return Some(match nh.center? {
                true => true,
                false => nh.count_neighbours(|frozen| !frozen.unwrap_or(true)) != 1,
            });
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

impl World {
    fn randomize_params(&mut self) {
        self.p0 = self.rng.gen();
    }
    pub fn run_and_evaluate(&mut self, p0: u8) -> f32 {
        self.p0 = p0;
        self.phase = 0;

        self.steps(3);
        self.frozen
            .iter_cells()
            .filter_map(|&frozen| {
                Some(match frozen? {
                    true => 0.,
                    false => 1.,
                })
            })
            .sum::<f32>()
    }
}

impl HexgridView for World {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        let cell = self.frozen.cell(pos)?;
        cell.map(|cell| CellView {
            cell_type: match cell {
                false => 0,
                true => 1,
            },
            ..Default::default()
        })
    }
    fn cell_text(&self, pos: coords::Cube) -> Option<String> {
        Some(match self.frozen.cell(pos)?? {
            true => "Frozen".into(),
            false => "Candidate".into(),
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
