/// Hex coordinate conversions
///
/// We are using "pointy topped" hexagons.
/// For conventions see: https://www.redblobgames.com/grids/hexagons/

pub type Cube = hex2d::Coordinate<i32>;
pub type Direction = hex2d::Direction;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct Offset {
    pub col: i32,
    pub row: i32,
}

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
    /// let pos: Cube = Offset{col: -1, row: -1}.into();
    /// assert_eq!(pos, Cube{x: 0, y: 1});
    /// ```
    fn from(p: Offset) -> Cube {
        // taken from redblob
        let x = p.col - (p.row - (p.row & 1)) / 2;
        let z = p.row;
        let y = -x - z;
        Cube::new(x, y)
    }
}

// impl From<Offset> for Axial {

pub trait ToIndex {
    fn to_index(&self) -> usize;
}

impl ToIndex for hex2d::Direction {
    fn to_index(&self) -> usize {
        // (hex2d docu specifies range [0, 6) but enforces a signed integer type)
        // maybe this can be somehow solved in the hex2d crate instead?
        self.to_int::<i32>() as usize
    }
}
