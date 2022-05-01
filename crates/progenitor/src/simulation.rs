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
    fn get_cell_text(&self, pos: coords::Cube) -> String {
        let cv = self.get_cell_view(pos);
        let mut lines = Vec::with_capacity(3);
        lines.push(format!("Type: {}", cv.cell_type));
        if let Some(e) = cv.energy {
            lines.push(format!("Energy: {}", e));
        }
        if let Some(dir) = cv.direction {
            lines.push(format!("Direction: {}", coords::compass_str(dir)));
        }
        lines.join("\n")
    }
    fn save_state(&self) -> Vec<u8>;
    fn load_state(&mut self, data: &[u8]);

    fn get_cells_rectangle(&self) -> Vec<CellView> {
        let pos = coords::Cube { x: 0, y: 0 };
        tile::iterate_rectangle(pos, tile::SIZE as i32, tile::SIZE as i32)
            .map(|coord| self.get_cell_view(coord))
            .collect()
    }
    // useful when called via trait object (allows loop unrolling)
    fn steps(&mut self, count: usize) {
        for _ in 0..count {
            self.step()
        }
    }
}

#[derive(Default, Serialize, Debug)]
pub struct CellView {
    pub cell_type: u8,
    pub energy: Option<u8>,
    pub direction: Option<Direction>,
}
