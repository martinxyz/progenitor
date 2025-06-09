use rand::Rng;

use crate::{bit_particles::BitParticles, Direction, Neighbourhood};
use Direction::*;

pub(crate) fn step(nh: Neighbourhood<BitParticles>) -> BitParticles {
    BitParticles::collect_neighbours(nh)
}

pub fn apply_air_rules(vapour: &mut BitParticles, rng: &mut impl Rng) {
    if rng.random::<u8>() < 200 {
        vapour.set_resting(1);
    } else {
        if vapour.resting() == 1 {
            vapour.set_resting(0);
        }
    }

    if rng.random::<u8>() < 3 {
        vapour.shuffle8_cheap(rng);
    }

    // strong downwards bias
    let mut o = vapour.outgoing();
    o = o.swapped(SouthWest, SouthEast); // straight down (zig-zag)
    o = o.swapped(NorthWest, NorthEast); // straight up

    if rng.random::<u8>() < 250 {
        if o.has(NorthEast) {
            o = o.swapped(NorthEast, East)
        } else if !o.has(SouthEast) {
            o = o.swapped(SouthEast, East)
        }
        if o.has(NorthWest) {
            o = o.swapped(NorthWest, West)
        } else if !o.has(SouthWest) {
            o = o.swapped(SouthWest, West)
        }
    }
    vapour.set_outgoing(o);
}
