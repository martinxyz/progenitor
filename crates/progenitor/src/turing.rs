use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::coords;
use crate::coords::Direction;
use crate::CellView;
use crate::HexgridView;
use crate::SimRng;
use crate::Simulation;
use crate::TorusTile;
use crate::VIEWPORT;

/* Based on the original "turing drawings":
https://github.com/maximecb/Turing-Drawings/blob/master/programs.js#L44-L48
N states, one start state (agent)
K symbols (grid cell types)
6 actions (hexgrid directions)
N x K -> N x K x A
 */
impl Turing {
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
pub struct Turing {
    pub grid: TorusTile<u8>,
    pos: coords::Cube,
    state: u8,
    rule_lut: Vec<Command>,
}

fn random_rule(rng: &mut impl Rng) -> Vec<Command> {
    (0..Turing::LUT_SIZE)
        .map(|_| Command {
            next_state: rng.random_range(0..Turing::STATES as u8),
            next_symbol: rng.random_range(0..Turing::SYMBOLS as u8),
            next_action: *Direction::all().choose(rng).unwrap(),
        })
        .collect()
}

impl Turing {
    pub fn new_with_seed(seed: u64) -> Turing {
        let mut rng = SimRng::seed_from_u64(seed);
        Turing {
            grid: TorusTile::new(0),
            rule_lut: random_rule(&mut rng),
            pos: Turing::CENTER.into(),
            state: 0,
        }
    }
    pub fn new() -> Turing {
        Self::new_with_seed(rand::rng().next_u64())
    }
}

impl Simulation for Turing {
    fn step(&mut self) {
        let command = {
            let symbol = self.grid.cell(self.pos);
            let key: usize = self.state as usize * Turing::SYMBOLS + symbol as usize;
            self.rule_lut[key]
        };

        self.grid.set_cell(self.pos, command.next_symbol);
        self.pos = self.pos + command.next_action;
        self.state = command.next_state;

        // if self.rng.random_bool(0.02) {
        //     self.rule_lut = random_rule(&mut self.rng);
        // }
    }

    fn save_state(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    fn load_state(&mut self, data: &[u8]) {
        *self = bincode::deserialize_from(data).unwrap();
    }
}

impl HexgridView for Turing {
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView> {
        Some(CellView {
            cell_type: if self.grid.is_same_pos(pos, self.pos) {
                0 // visualize position of the turing head
            } else {
                self.grid.cell(pos) + 1
            },
            ..Default::default()
        })
    }

    fn viewport_hint(&self) -> coords::Rectangle {
        VIEWPORT
    }
}

impl Default for Turing {
    fn default() -> Self {
        Self::new()
    }
}
