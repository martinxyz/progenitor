//! Potts model
//! See https://en.wikipedia.org/wiki/Cellular_Potts_model#Model_description

use rand::seq::SliceRandom;

use crate::coords::Direction;
use crate::{AxialTile, Neighbourhood, SimRng};

pub trait PottsRule {
    type Cell: Copy + Eq;
    fn energy(&self, neighbourhood: Neighbourhood<Self::Cell>) -> f32;
}

pub fn step_axial<Rule: PottsRule>(
    tile: &AxialTile<Rule::Cell>,
    border: Rule::Cell,
    rule: &Rule,
    rng: &mut SimRng,
) -> AxialTile<Rule::Cell> {
    let mut tile = tile.clone();

    let steps = tile.area() / 16;

    for _ in 0..steps {
        let pos = tile.random_pos(rng);
        if let Some(nh) = tile.neighbourhood(pos) {
            let dir = *Direction::all().choose(rng).unwrap();
            let candidate = nh.neighbour(dir);
            if nh.center == border || candidate == border {
                continue
            }
            let energy = rule.energy(nh);
            let energy2 = rule.energy(Neighbourhood {
                center: candidate,
                neighbours: nh.neighbours,
            });
            let delta = energy2 - energy;
            // let one_over_T = 1. / 20.f;
            // if delta < 0 || rng.gen_bool(f32::exp(-delta*one_over_T)) {
            if delta < 0.0 {
                tile.set_cell(pos, candidate);
            }
        }
    }
    tile
}
