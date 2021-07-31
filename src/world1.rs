use std::cmp::max;

use rand::Rng;

use crate::{CellType, CellTypeRef, cell::CellTypes};

#[derive(Debug)]
pub struct Params {
    genesis_grow_p: u8,
    pre_wall_grow_p: u8,
}

fn mut_u8_p128(p: u8, rng: &mut impl Rng) -> u8 {
    let step = max(1, p / 8) as i8;
    (p as i16 + rng.gen_range(-step..=step) as i16).clamp(0, 128) as u8
}

fn mut_grow_p(p: u8, rng: &mut impl Rng) -> u8 {
    if rng.gen_bool(0.1) {  // type-changing mutation (note: should use enum for that...)
        match rng.gen_range(0..=6) {
            0 => 128,  // grow always in all directions
            1 => 129,  // grow always with current heading
            2 => 255,  // grow always with random heading
            _ => rng.gen_range(0..=127),  // grow with probability (note: should not be uniform...)
        }
    } else if p < 128 {
        mut_u8_p128(p, rng)
    } else {
        p
    }
}

impl Params {
    pub fn mutate(&mut self, rng: &mut impl Rng) {
        self.genesis_grow_p = mut_grow_p(self.genesis_grow_p, rng);
        self.pre_wall_grow_p = mut_grow_p(self.pre_wall_grow_p, rng);
    }
}

impl Default for Params {
   fn default() -> Self {
        Self {
            genesis_grow_p: 2,
            pre_wall_grow_p: 80,
        }
    }
}

pub fn rules(params: &Params) -> CellTypes {
    let mut types = CellTypes::new();

    let mut ref_iterator = (0..255u8).map(CellTypeRef);
    let mut new_ref = || ref_iterator.next().expect("too many cell types");

    let genesis = new_ref();
    let air = new_ref();
    let wall = new_ref();
    let pre_wall = new_ref();

    types[genesis] = CellType {
        priority: -128,
        transform_at_random_p: 128,
        transform_into: air,
        grow_child_type: pre_wall,
        grow_p: params.genesis_grow_p,
        ..CellType::default()
    };

    types[air] = CellType {
        priority: -1,
        ..CellType::default()
    };

    types[pre_wall] = CellType {
        priority: 19,
        // initial_energy: 2,
        transform_at_random_p: 128,
        transform_into: wall,
        grow_child_type: wall,
        grow_p: params.pre_wall_grow_p,
        ..CellType::default()
    };
    types[wall] = CellType {
        priority: 20,
        // initial_energy: 2,
        ..CellType::default()
    };
    types
}
