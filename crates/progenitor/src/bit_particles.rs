#![allow(dead_code)]

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{DirectionSet, Neighbourhood};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct BitParticles(u8);

impl BitParticles {
    pub const EMPTY: Self = BitParticles(0b00000000);
    // pub const EIGHT: Self = BitParticles(0b11111111);
    // pub fn random50(rng: &mut impl Rng) -> Self {
    //     Self(rng.gen())
    // }
    pub fn count(&self) -> u8 {
        self.0.count_ones() as u8
    }
    pub fn outgoing(&self) -> DirectionSet {
        DirectionSet::from_bits(self.0 & 0b00111111)
    }
    pub fn resting(&self) -> u8 {
        (self.0 & 0b11000000).count_ones() as u8
    }
    pub fn ca_step(nh: Neighbourhood<Self>) -> Self {
        let incoming = DirectionSet::matching(|dir| nh[-dir].outgoing().contains(dir));
        let resting = nh.center.0 & 0b11000000;
        Self(resting | incoming.bits())
    }
    pub fn set_outgoing(&mut self, outgoing: DirectionSet) {
        self.0 &= 0b11000000;
        self.0 |= outgoing.bits();
    }
    pub fn set_resting(&mut self, resting: u8) {
        self.0 &= 0b00111111;
        self.0 |= match resting {
            0 => 0b00000000,
            1 => 0b01000000,
            2 => 0b11000000,
            _ => unreachable!("cannot have more than two resting particles"),
        }
    }
    pub fn reflect_all(&mut self) {
        self.set_outgoing(self.outgoing().mirrored());
    }
    pub fn shuffle8_cheap(&mut self, rng: &mut impl Rng) {
        let mut rnd: u8 = rng.gen();
        let idx1 = rnd & 0b111;
        rnd >>= 4;
        let idx2 = rnd & 0b111;
        let bit1 = (self.0 & (1 << idx1)) != 0;
        let bit2 = (self.0 & (1 << idx2)) != 0;

        if bit1 != bit2 {
            self.0 ^= (1 << idx1) | (1 << idx2);
        }
    }
    pub fn shuffle8_cheap_4x(&mut self, rng: &mut impl Rng) {
        let mut rnd: u32 = rng.gen();
        for _ in 0..4 {
            let idx1 = (rnd as u8) & 0b111;
            rnd >>= 4;
            let idx2 = (rnd as u8) & 0b111;
            rnd >>= 4;
            let bit1 = (self.0 & (1 << idx1)) != 0;
            let bit2 = (self.0 & (1 << idx2)) != 0;

            if bit1 != bit2 {
                self.0 ^= (1 << idx1) | (1 << idx2);
            }
        }
    }
    pub fn swap_random_neighbours(&mut self, rng: &mut impl Rng) {
        let idx1 = ((rng.gen::<u32>() as u64 * 6) >> 32) as u8;
        let idx2 = if idx1 == 5 { 0 } else { idx1 + 1 };

        let bit1 = (self.0 & (1 << idx1)) != 0;
        let bit2 = (self.0 & (1 << idx2)) != 0;
        if bit1 != bit2 {
            self.0 ^= (1 << idx1) | (1 << idx2);
        }
    }
}

impl Debug for BitParticles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08b}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::SimRng;
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn test_shuffle_cheap() {
        let mut rng = SimRng::seed_from_u64(42);
        for i in 0..255 {
            let mut p = BitParticles(i);
            let count_before = p.count();
            p.shuffle8_cheap(&mut rng);
            p.shuffle8_cheap_4x(&mut rng);
            let count_after = p.count();
            assert_eq!(count_before, count_after);
        }
    }
}
