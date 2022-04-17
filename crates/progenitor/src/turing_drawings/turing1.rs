use hex2d::Direction;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::tile;
use crate::tile::Tile;
use crate::CellView;
use crate::Simulation;

const INPUT_BITS: usize = 7; // 6 neighbours + center
const STATE_BITS: usize = 5; // free parameter (max 8)
const LUT_SIZE: usize = 1 << (STATE_BITS + INPUT_BITS);

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Command {
    next_value: bool,
    next_move: Direction,
    next_state: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Turing1 {
    alive: Tile<bool>,
    pos: coords::Cube,
    state: u8,
    rule_lut: Vec<Command>, // uh, I didn't want to serialize() this for every step() snapshot... but it's needed since it's mutable
    rng: Pcg32,
}

fn random_rule(rng: &mut impl Rng) -> Vec<Command> {
    (0..LUT_SIZE)
        .map(|_| Command {
            next_value: rng.gen(),
            next_move: *Direction::all().choose(rng).unwrap(),
            next_state: rng.gen::<u8>() >> (8 - STATE_BITS),
        })
        .collect()
}

impl Turing1 {
    pub fn new() -> Turing1 {
        let mut rng = Pcg32::from_rng(thread_rng()).unwrap();
        Turing1 {
            alive: Tile::new(false),
            rule_lut: random_rule(&mut rng),
            pos: coords::Offset { col: 5, row: 5 }.into(),
            state: 0,
            rng,
        }
    }
}

impl Simulation for Turing1 {
    fn step(&mut self) {
        let command = {
            let center = self.alive.get_cell(self.pos);
            let neighbours = self.alive.get_neighbours(self.pos).map(|(_, cell)| cell);
            let mut key: usize = if center { 1 << 6 } else { 0 };
            for (i, cell) in neighbours.into_iter().enumerate() {
                if cell {
                    key |= 1 << i;
                }
            }
            key |= (self.state as usize) << INPUT_BITS;
            self.rule_lut[key]
        };

        self.alive.set_cell(self.pos, command.next_value);
        self.pos = self.pos + command.next_move;
        self.state = command.next_state;

        // if self.rng.gen_bool(0.002) {
        //     self.rule_lut = random_rule(&mut self.rng);
        // }
    }

    // FIXME: duplicated code (same for every Simulation using Tile storage)
    fn get_cells_rectangle(&self) -> Vec<CellView> {
        let pos = coords::Cube { x: 0, y: 0 };
        tile::iterate_rectangle(pos, tile::SIZE as i32, tile::SIZE as i32)
            .map(|coord| self.get_cell_view(coord))
            .collect()
    }

    fn get_cell_view(&self, pos: coords::Cube) -> CellView {
        CellView {
            cell_type: if self.alive.is_same_pos(pos, self.pos) {
                2 // visualize position of the turing head
            } else {
                match self.alive.get_cell(pos) {
                    false => 0,
                    true => 1,
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

impl Default for Turing1 {
    fn default() -> Self {
        Self::new()
    }
}
