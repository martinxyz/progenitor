use hex2d::Direction;
use serde::Serialize;

use crate::{coords, tile};

pub trait Simulation {
    fn step(&mut self);
    // Note to self: do not try again to optimize this by returning a trait like
    // "impl Iterator". It may not be fully impossible, but the effort is
    // completely wasted. This API is for rendering; an extra copy or five of
    // this Vec<SmallThing> will be neglegible compared to rendering.
    fn get_cell_view(&self, pos: coords::Cube) -> CellView;
    fn save_state(&self) -> Vec<u8>;
    fn load_state(&mut self, data: &[u8]);

    fn get_cells_rectangle(&self) -> Vec<CellView> {
        let pos = coords::Cube { x: 0, y: 0 };
        tile::iterate_rectangle(pos, tile::SIZE as i32, tile::SIZE as i32)
            .map(|coord| self.get_cell_view(coord))
            .collect()
    }
}

#[derive(Default, Serialize)]
pub struct CellView {
    pub cell_type: u8,
    pub energy: Option<u8>,
    pub direction: Option<Direction>,
}
