use std::cmp::max;

use rand::Rng;

use crate::{cell::CellTypes, CellType, CellTypeRef, GrowDirection};

#[derive(Debug)]
pub struct Params {
    genesis_grow_p: u8,
    genesis_grow_dir: GrowDirection,
    pre_wall_grow_p: u8,
    pre_wall_grow_dir: GrowDirection,
}

fn mutate_u8_p128(p: u8, rng: &mut impl Rng) -> u8 {
    let step = max(1, p / 8) as i8;
    (p as i16 + rng.gen_range(-step..=step) as i16).clamp(0, 128) as u8
}

fn mutate_grow(p: &mut u8, dir: &mut GrowDirection, rng: &mut impl Rng) {
    if rng.gen_bool(0.1) {
        // type-changing mutation
        *dir = match rng.gen_range(0..4) {
            0 => GrowDirection::Forward,
            1 => GrowDirection::RandomChoice,
            _ => GrowDirection::All,
        };
        *p = match rng.gen_range(0..4) {
            0 => 0,
            1 => *p,
            _ => rng.gen_range(0..=127), // (note: should not be uniform...)
        }
    } else {
        *p = mutate_u8_p128(*p, rng)
    }
}

impl Params {
    pub fn mutate(&mut self, rng: &mut impl Rng) {
        mutate_grow(&mut self.genesis_grow_p, &mut self.genesis_grow_dir, rng);
        mutate_grow(&mut self.pre_wall_grow_p, &mut self.pre_wall_grow_dir, rng);
    }
}

impl Default for Params {
    fn default() -> Self {
        Self {
            genesis_grow_p: 2,
            genesis_grow_dir: GrowDirection::All,
            pre_wall_grow_p: 80,
            pre_wall_grow_dir: GrowDirection::All,
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
