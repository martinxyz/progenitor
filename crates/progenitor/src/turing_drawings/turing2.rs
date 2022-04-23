use hex2d::Direction;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;
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
const STATES: usize = 5; // states of agent
const SYMBOLS: usize = 4; // distinct cell types
const LUT_SIZE: usize = STATES * SYMBOLS;

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Command {
    next_state: u8,
    next_symbol: u8,
    next_action: Direction,
}

#[derive(Serialize, Deserialize)]
pub struct Turing2 {
    grid: Tile<u8>,
    pos: coords::Cube,
    state: u8,
    rule_lut: Vec<Command>, // uh, I didn't want to serialize() this for every step() snapshot... but it's needed since it's mutable
}

fn random_rule(rng: &mut impl Rng) -> Vec<Command> {
    (0..LUT_SIZE)
        .map(|_| Command {
            next_state: rng.gen_range(0..STATES as u8),
            next_symbol: rng.gen_range(0..SYMBOLS as u8),
            next_action: *Direction::all().choose(rng).unwrap(),
        })
        .collect()
}

impl Turing2 {
    pub fn new() -> Turing2 {
        let mut rng = Pcg32::from_rng(thread_rng()).unwrap();
        Turing2 {
            grid: Tile::new(0),
            rule_lut: random_rule(&mut rng),
            pos: coords::Offset { col: 5, row: 5 }.into(),
            state: 0,
        }
    }
}

impl Simulation for Turing2 {
    fn step(&mut self) {
        let command = {
            let symbol = self.grid.get_cell(self.pos);
            let key: usize = self.state as usize * symbol as usize;
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
