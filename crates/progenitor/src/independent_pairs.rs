use rand::prelude::*;

use crate::coords::Direction;
use crate::{AxialTile, Neighbourhood, SimRng};

pub trait PairRule {
    type Cell: Copy + Eq;
    fn pair_rule(
        &self,
        source: Neighbourhood<Self::Cell>,
        target: Neighbourhood<Self::Cell>,
        direction: Direction,
    ) -> (Self::Cell, Self::Cell);
}

pub fn step_axial<Rule: PairRule>(
    tile: &AxialTile<Rule::Cell>,
    border: Rule::Cell,
    rule: &Rule,
    rng: &mut SimRng,
) -> AxialTile<Rule::Cell> {
    let mut tile = tile.clone();

    let steps = tile.area() / 16;

    for _ in 0..steps {
        let source_pos = tile.random_pos(rng);
        let direction = *Direction::all().choose(rng).unwrap();
        let target_pos = source_pos + direction;
        let (Some(source), Some(target)) = (
            tile.neighbourhood(source_pos),
            tile.neighbourhood(target_pos),
        ) else {
            continue;
        };
        if source.center == border || target.center == border {
            continue;
        }
        let (source, target) = rule.pair_rule(source, target, direction);
        tile.set_cell(source_pos, source);
        tile.set_cell(target_pos, target);
    }
    tile
}
