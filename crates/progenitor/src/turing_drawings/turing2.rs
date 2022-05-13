use hex2d::Direction;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::tile::Tile;
use crate::CellView;
use crate::Simulation;

/* Based on the original "turing drawings":
https://github.com/maximecb/Turing-Drawings/blob/master/programs.js#L44-L48
N states, one start state (agent)
K symbols (grid cell types)
6 actions (hexgrid directions)
N x K -> N x K x A
 */
impl Turing2 {
    pub const STATES: usize = 5; // states of agent
    pub const SYMBOLS: usize = 4; // distinct cell types
    pub const LUT_SIZE: usize = Self::STATES * Self::SYMBOLS;
    pub const CENTER: coords::Offset = coords::Offset { col: 5, row: 5 };
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Command {
    next_state: u8,
    next_symbol: u8,
    next_action: Direction,
}

#[derive(Serialize, Deserialize)]
pub struct Turing2 {
    pub grid: Tile<u8>,
    pos: coords::Cube,
    state: u8,
    rule_lut: Vec<Command>,
}

fn random_rule(rng: &mut impl Rng) -> Vec<Command> {
    (0..Turing2::LUT_SIZE)
        .map(|_| Command {
            next_state: rng.gen_range(0..Turing2::STATES as u8),
            next_symbol: rng.gen_range(0..Turing2::SYMBOLS as u8),
            next_action: *Direction::all().choose(rng).unwrap(),
        })
        .collect()
}

impl Turing2 {
    pub fn new_with_seed(seed: u64) -> Turing2 {
        let mut rng = Pcg32::seed_from_u64(seed);
        Turing2 {
            grid: Tile::new(0),
            rule_lut: random_rule(&mut rng),
            pos: Turing2::CENTER.into(),
            state: 0,
        }
    }
    pub fn new() -> Turing2 {
        Self::new_with_seed(thread_rng().next_u64())
    }
}

impl Simulation for Turing2 {
    fn step(&mut self) {
        let command = {
            let symbol = self.grid.get_cell(self.pos);
            let key: usize = self.state as usize * Turing2::SYMBOLS + symbol as usize;
            self.rule_lut[key]
        };

        self.grid.set_cell(self.pos, command.next_symbol);
        self.pos = self.pos + command.next_action;
        self.state = command.next_state;

        // if self.rng.gen_bool(0.02) {
        //     self.rule_lut = random_rule(&mut self.rng);
        // }
    }

    fn get_cell_view(&self, pos: coords::Cube) -> CellView {
        CellView {
            cell_type: if self.grid.is_same_pos(pos, self.pos) {
                0 // visualize position of the turing head
            } else {
                self.grid.get_cell(pos) + 1
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

impl Default for Turing2 {
    fn default() -> Self {
        Self::new()
    }
}
