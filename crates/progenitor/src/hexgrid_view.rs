use hex2d::Direction;
use serde::Serialize;

use crate::{coords, SIZE};

/// API for rendering a single hex cell.
/// The meaning of fields depend on the simulation.
#[derive(Default, Serialize, Debug)]
pub struct CellView {
    /// Category. Rendered as discrete colors, usually 0 means empty/air.
    pub cell_type: u8,
    /// A scalar value, rendered as brightness, 0 for dark.
    pub energy: Option<u8>,
    /// For agent heading or growth direction. Rendered as small dot.
    pub direction: Option<Direction>,
}

/// API for hexgrid rendering and inspection (e.g. in the web UI).
pub trait HexgridView {
    // Note to self: do not try again to optimize this by returning a trait like
    // "impl Iterator". It may not be fully impossible, but the effort is
    // completely wasted. This API is for rendering; an extra copy or five of
    // this Vec<SmallThing> will be neglegible compared to rendering.
    fn cell_view(&self, pos: coords::Cube) -> Option<CellView>;
    fn cell_text(&self, pos: coords::Cube) -> Option<String> {
        let cv = self.cell_view(pos)?;
        let mut lines = Vec::with_capacity(3);
        lines.push(format!("Type: {}", cv.cell_type));
        if let Some(e) = cv.energy {
            lines.push(format!("Energy: {}", e));
        }
        if let Some(dir) = cv.direction {
            lines.push(format!("Direction: {}", coords::compass_str(dir)));
        }
        Some(lines.join("\n"))
    }

    fn cells_rectangle(&self) -> Vec<CellView> {
        let pos = coords::Cube { x: 0, y: 0 };
        coords::iterate_rectangle(pos, SIZE as i32, SIZE as i32)
            .map(|coord| {
                self.cell_view(coord).unwrap_or(CellView {
                    cell_type: 255,
                    ..Default::default()
                })
            })
            .collect()
    }
}
