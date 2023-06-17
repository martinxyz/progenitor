//! Cellular Automata Helpers

use hex2d::Direction;

use crate::TorusTile;

pub struct Neighbourhood<Cell: Copy> {
    pub center: Cell,
    pub neighbours: [(Direction, Cell); 6],
}

pub fn step<Cell: Copy>(
    tile: &TorusTile<Cell>,
    rule: impl FnMut(Neighbourhood<Cell>) -> Cell,
) -> TorusTile<Cell> {
    // optimize: depending o `f`, each step may depend on the previous step's
    // data through the RNG state, independent RNGs may be better (or no RNGs)
    tile.iter_radius_1()
        .map(|(center, neighbours)| Neighbourhood { center, neighbours })
        .map(rule)
        .collect()
}
