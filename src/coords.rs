//! Hex coordinate conversions
//!
//! All hexagons are "pointy topped".
//!
//! Conventions follow the redblobgames
//! [hexagons](https://www.redblobgames.com/grids/hexagons/) article.

/// Cube coordinates
///
/// [Cube coordinates](https://www.redblobgames.com/grids/hexagons/#coordinates-cube) are used for general hexagonal algorithms.
///
/// Re-exported from the [hex2d](https://crates.io/crates/hex2d) crate:
///
pub use hex2d::Coordinate as Cube;

/// One of the 6 directions
///
/// Re-exported from the [hex2d](https://crates.io/crates/hex2d) crate:
///
pub use hex2d::Direction;

/// Offset coordinates
///
/// [Offset coordinates](https://www.redblobgames.com/grids/hexagons/#coordinates-offset)
/// are used to render a rectangular view.
///
/// We only support "odd-r" (as defined in the link above).
///
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct Offset {
    pub col: i32,
    pub row: i32,
}

/// Axial coordinates
///
/// We use [axial coordinates](https://www.redblobgames.com/grids/hexagons/#coordinates-axial) to iterate over [map storage](https://www.redblobgames.com/grids/hexagons/#map-storage).
///
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct Axial {
    pub q: i32,
    pub r: i32,
}

impl Offset {
    pub fn new(col: i32, row: i32) -> Self {
        Self { col, row }
    }
}

impl Axial {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }
}

impl From<Offset> for Cube {
    // This really doesn't exist in hex2d :-/
    // (it exists in "chickenwire", but...)
    /// Convert from offset coordinates (odd-r)
    ///
    /// ```
    /// use progenitor::coords::{Cube, Offset};
    /// let pos: Cube = Offset { col: -1, row: -1 }.into();
    /// assert_eq!(pos, Cube { x: 0, y: 1 });
    /// ```
    fn from(p: Offset) -> Cube {
        // taken from redblob
        let x = p.col - (p.row - (p.row & 1)) / 2;
        let z = p.row;
        let y = -x - z;
        Cube { x, y }
    }
}

// impl From<Offset> for Axial {
