//! Hex coordinate conversions
//!
//! All hexagons are "pointy topped".
//!
//! Conventions follow the redblobgames
//! [hexagons](https://www.redblobgames.com/grids/hexagons/) article.

use serde::{Deserialize, Serialize};

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

pub fn compass_str(dir: Direction) -> &'static str {
    match dir {
        Direction::YZ => "NW",
        Direction::XZ => "NE",
        Direction::XY => "E",
        Direction::ZY => "SE",
        Direction::ZX => "SW",
        Direction::YX => "W",
    }
}

/// A set of directions
///
/// It can be empty or contain at most all 6 directions.
///
/// # Examples
///
/// ```
/// use progenitor::{DirectionSet, Direction};
///
/// let all = DirectionSet::all();
/// assert!(all.contains(Direction::XZ));
///
/// let none = DirectionSet::none();
/// assert!(!none.contains(Direction::XZ));
///
/// let single = DirectionSet::single(Direction::XZ);
/// assert!(single.contains(Direction::XZ));
/// assert!(!single.contains(Direction::XY));
///
/// let some = DirectionSet::matching(|d| d == Direction::XY || d == Direction::YZ);
/// assert!(some.contains(Direction::XY));
/// assert!(some.contains(Direction::YZ));
/// assert!(!some.contains(Direction::XZ));
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
// #[serde(transparent)]
pub struct DirectionSet {
    mask: u8,
}

impl DirectionSet {
    pub fn none() -> Self {
        DirectionSet { mask: 0 }
    }
    pub fn all() -> Self {
        DirectionSet { mask: (1 << 6) - 1 }
    }
    pub fn single(dir: Direction) -> Self {
        DirectionSet {
            mask: 1 << (dir as u8),
        }
    }
    pub fn matching<F>(mut contains: F) -> Self
    where
        F: FnMut(Direction) -> bool,
    {
        let mask = (0..6)
            .filter(|i| contains(Direction::from_int(*i)))
            .fold(0, |mask, i| mask | (1 << i));
        DirectionSet { mask }
    }
    pub fn contains(&self, dir: Direction) -> bool {
        let dir_mask = 1 << (dir as u8);
        self.mask & dir_mask != 0
    }
    #[must_use]
    pub fn with(&self, dir: Direction, present: bool) -> Self {
        DirectionSet {
            mask: if present {
                self.mask | (1 << (dir as u8))
            } else {
                self.mask & !(1 << (dir as u8))
            },
        }
    }
    #[must_use]
    pub fn mirrored(&self) -> Self {
        DirectionSet::matching(|dir| self.contains(-dir))
    }
    pub fn count(&self) -> u8 {
        self.mask.count_ones() as u8
    }
}

/* doesn't seem to work, maybe because &[Direction] is a different type from &[Direction, 2]?
impl From<&[Direction]> for DirectionSet {
    fn from(directions: &[Direction]) -> Self {
        DirectionSet{
            mask: directions.iter().fold(0, |mask, dir| {
                mask | (1 << (*dir as u8))
            })
        }
    }
}
*/

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

/* currently unused
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
*/

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

impl From<Cube> for Offset {
    /// Convert to offset coordinates (odd-r)
    ///
    /// ```
    /// use progenitor::coords::{Cube, Offset};
    /// let pos: Offset = Cube { x: 0, y: 1 }.into();
    /// assert_eq!(pos, Offset { col: -1, row: -1 });
    /// ```
    fn from(p: Cube) -> Self {
        let col = p.x + (p.z() - (p.z() & 1)) / 2;
        let row = p.z();
        Offset { col, row }
    }
}

// impl From<Offset> for Axial {

/// A rectangle, used for rendering in offset coordinates.
#[derive(Clone, Copy)]
pub struct Rectangle {
    pub pos: Cube,
    pub width: i32,
    pub height: i32,
}

/// Iterator over a rectangle in offset coordinates.
pub fn iterate_rectangle(rect: Rectangle) -> impl Iterator<Item = Cube> {
    (0..rect.height)
        .flat_map(move |row| (0..rect.width).map(move |col| rect.pos + Offset { col, row }))
}
