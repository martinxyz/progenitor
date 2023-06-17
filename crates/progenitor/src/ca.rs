//! Cellular Automata Helpers

use hex2d::Direction;

use crate::coords;
use crate::TorusTile;

pub fn step<Cell: Copy>(
    tile: &TorusTile<Cell>,
    mut rule: impl FnMut(Cell, Neighbours<Cell>) -> Cell,
) -> TorusTile<Cell> {
    // optimize: depending o `f`, each step may depend on the previous step's
    // data through the RNG state, independent RNGs may be better (or no RNGs)
    tile.iter_radius_1()
        .map(|(center, neighbours)| rule(center, Neighbours(neighbours)))
        .collect()
}

pub struct Neighbours<Cell: Copy>([(Direction, Cell); 6]);

impl<Cell: Copy> Neighbours<Cell> {
    pub fn iter(&self) -> impl Iterator<Item = (Direction, Cell)> {
        self.0.into_iter()
    }
}
