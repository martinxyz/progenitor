//! Hex coordinate conversions
//!
//! All hexagons are "pointy topped". We use axial coordinates (q, r).
//! Incrementing q moves right (east). Incrementing r moves buttom-down (south-east).
//!
//! Conventions follow the redblobgames
//! [hexagons](https://www.redblobgames.com/grids/hexagons/) article.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Index, Sub};

/// Cube coordinates
///
/// [Cube coordinates](https://www.redblobgames.com/grids/hexagons/#coordinates-cube) are used for general hexagonal algorithms.
///
/// Re-exported from the [hex2d](https://crates.io/crates/hex2d) crate:
///
pub use hex2d::Coordinate as Cube;

/// One of the 6 directions
///
/// Compass names, assuming pointy-top hexagons.
///
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Direction {
    NorthWest,
    NorthEast,
    East,
    SouthEast,
    SouthWest,
    West,
}

const ALL_DIRECTIONS: [Direction; 6] = [
    Direction::NorthWest,
    Direction::NorthEast,
    Direction::East,
    Direction::SouthEast,
    Direction::SouthWest,
    Direction::West,
];

impl From<Direction> for hex2d::Direction {
    fn from(value: Direction) -> Self {
        match value {
            Direction::NorthWest => hex2d::Direction::YZ,
            Direction::NorthEast => hex2d::Direction::XZ,
            Direction::East => hex2d::Direction::XY,
            Direction::SouthEast => hex2d::Direction::ZY,
            Direction::SouthWest => hex2d::Direction::ZX,
            Direction::West => hex2d::Direction::YX,
        }
    }
}

impl From<hex2d::Direction> for Direction {
    fn from(value: hex2d::Direction) -> Self {
        match value {
            hex2d::Direction::YZ => Direction::NorthWest,
            hex2d::Direction::XZ => Direction::NorthEast,
            hex2d::Direction::XY => Direction::East,
            hex2d::Direction::ZY => Direction::SouthEast,
            hex2d::Direction::ZX => Direction::SouthWest,
            hex2d::Direction::YX => Direction::West,
        }
    }
}

impl Add<Direction> for hex2d::Coordinate {
    type Output = Self;
    fn add(self, rhs: Direction) -> Self::Output {
        self + hex2d::Direction::from(rhs)
    }
}
impl Sub<Direction> for hex2d::Coordinate {
    type Output = Self;
    fn sub(self, rhs: Direction) -> Self::Output {
        self + hex2d::Direction::from(-rhs)
    }
}

// note: hex2d::Direction doesn't implement Sub
impl Add<hex2d::Angle> for Direction {
    type Output = Direction;
    fn add(self, rhs: hex2d::Angle) -> Self::Output {
        hex2d::Direction::add(self.into(), rhs).into()
    }
}

impl TryFrom<i32> for Direction {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == Direction::NorthWest as i32 => Ok(Direction::NorthWest),
            x if x == Direction::NorthEast as i32 => Ok(Direction::NorthEast),
            x if x == Direction::East as i32 => Ok(Direction::East),
            x if x == Direction::SouthEast as i32 => Ok(Direction::SouthEast),
            x if x == Direction::SouthWest as i32 => Ok(Direction::SouthWest),
            x if x == Direction::West as i32 => Ok(Direction::West),
            _ => Err(()),
        }
    }
}

impl Direction {
    pub fn all() -> [Direction; 6] {
        ALL_DIRECTIONS
    }

    pub fn name_short(&self) -> &str {
        match self {
            Direction::NorthWest => "NW",
            Direction::NorthEast => "NE",
            Direction::East => "E",
            Direction::SouthEast => "SE",
            Direction::SouthWest => "SW",
            Direction::West => "W",
        }
    }
    pub fn name_long(&self) -> &str {
        match self {
            Direction::NorthWest => "NorthWest",
            Direction::NorthEast => "NorthEast",
            Direction::East => "East",
            Direction::SouthEast => "SouthEast",
            Direction::SouthWest => "SouthWest",
            Direction::West => "West",
        }
    }

    pub fn from_int(i: i32) -> Direction {
        i.rem_euclid(6).try_into().unwrap()
    }
}

impl std::ops::Neg for Direction {
    type Output = Direction;
    fn neg(self) -> Direction {
        match self {
            Direction::NorthWest => Direction::SouthEast,
            Direction::NorthEast => Direction::SouthWest,
            Direction::East => Direction::West,
            Direction::SouthEast => Direction::NorthWest,
            Direction::SouthWest => Direction::NorthEast,
            Direction::West => Direction::East,
        }
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
/// assert!(all.contains(Direction::NorthEast));
///
/// let none = DirectionSet::none();
/// assert!(!none.contains(Direction::NorthEast));
///
/// let single = DirectionSet::single(Direction::NorthEast);
/// assert!(single.contains(Direction::NorthEast));
/// assert!(!single.contains(Direction::East));
///
/// let some = DirectionSet::matching(|d| d == Direction::East || d == Direction::NorthWest);
/// assert!(some.contains(Direction::East));
/// assert!(some.contains(Direction::NorthWest));
/// assert!(!some.contains(Direction::NorthEast));
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
// #[serde(transparent)]
pub struct DirectionSet {
    mask: u8,
}

impl DirectionSet {
    pub const fn none() -> Self {
        DirectionSet { mask: 0b00000000 }
    }
    pub const fn all() -> Self {
        DirectionSet { mask: 0b00111111 }
    }
    pub fn from_bits(mask: u8) -> Self {
        assert_eq!(mask & 0b11000000, 0);
        Self { mask }
    }
    pub fn bits(&self) -> u8 {
        self.mask
    }
    pub const fn single(dir: Direction) -> Self {
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
    #[must_use]
    pub fn transmuted(&self, transmute: impl Fn(Direction) -> Direction) -> Self {
        DirectionSet::matching(|dir| self.contains(transmute(dir)))
    }
    #[must_use]
    pub fn swapped(&self, dir1: Direction, dir2: Direction) -> Self {
        let has1 = self.contains(dir1);
        let has2 = self.contains(dir2);
        self.with(dir1, has2).with(dir2, has1)
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

#[derive(Clone, Copy)]
pub struct Neighbourhood<T: Copy> {
    pub center: T,
    pub neighbours: [T; 6],
}

impl<T: Copy> Neighbourhood<T> {
    pub fn iter_dirs(&self) -> impl Iterator<Item = (Direction, T)> {
        Direction::all().into_iter().zip(self.neighbours)
    }

    pub fn neighbour(&self, dir: Direction) -> T {
        self.neighbours[dir as usize]
    }

    pub fn count_neighbours(&self, condition: impl Fn(T) -> bool) -> i32 {
        self.neighbours
            .into_iter()
            .filter(|&n| condition(n))
            .count()
            .try_into()
            .unwrap()
    }

    // actually, there is slice .rotate_left ...
    // pub fn neighbours_rotated_right(&self) -> [T; 6] {

    // pub fn tripods(&self) -> impl Iterator<Item = Tripod<T>> {
    //     let n = self.neighbours;
    //     (0..6).map(move |i| Tripod {
    //         left: n[(i + 5) % 6],
    //         forward: n[i],
    //         right: n[(i + 1) % 6],
    //     })
    // }

    pub fn map<T2: Copy>(&self, f: impl Fn(T) -> T2) -> Neighbourhood<T2> {
        Neighbourhood {
            center: f(self.center),
            neighbours: self.neighbours.map(f),
        }
    }
}

impl<T: Copy> Neighbourhood<Option<T>> {
    pub fn valid_only(&self) -> Option<Neighbourhood<T>> {
        Some(Neighbourhood {
            center: self.center?,
            neighbours: [
                self.neighbours[0]?,
                self.neighbours[1]?,
                self.neighbours[2]?,
                self.neighbours[3]?,
                self.neighbours[4]?,
                self.neighbours[5]?,
            ],
        })
    }
}

impl<T: Copy> Index<Direction> for Neighbourhood<T> {
    type Output = T;

    fn index(&self, index: Direction) -> &Self::Output {
        &self.neighbours[index as usize]
    }
}
