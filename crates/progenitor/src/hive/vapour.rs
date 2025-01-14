use rand::Rng;

use crate::{bit_particles::BitParticles, Direction, Neighbourhood};
use Direction::*;

pub(crate) fn step(nh: Neighbourhood<BitParticles>) -> BitParticles {
    BitParticles::collect_neighbours(nh)
}

pub fn apply_air_rules(vapour: &mut BitParticles, rng: &mut impl Rng) {
    if rng.gen::<u8>() < 200 {
        vapour.set_resting(1);
    } else {
        if vapour.resting() == 1 {
            vapour.set_resting(0);
        }
    }

    if rng.gen::<u8>() < 3 {
        vapour.shuffle8_cheap(rng);
    }

    // strong downwards bias
    let mut o = vapour.outgoing();
    o = o.swapped(SouthWest, SouthEast); // straight down (zig-zag)
    o = o.swapped(NorthWest, NorthEast); // straight up

    if rng.gen::<u8>() < 250 {
        if o.contains(NorthEast) {
            o = o.swapped(NorthEast, East)
        } else if !o.contains(SouthEast) {
            o = o.swapped(SouthEast, East)
        }
        if o.contains(NorthWest) {
            o = o.swapped(NorthWest, West)
        } else if !o.contains(SouthWest) {
            o = o.swapped(SouthWest, West)
        }
    }
    vapour.set_outgoing(o);
}
