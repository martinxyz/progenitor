use crate::{coords, Neighbourhood};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;

use coords::Direction;

const SIZE_LOG2: u32 = 5;
/// Tile storage is a SIZE x SIZE parallelogram.
pub const SIZE: u32 = 1 << SIZE_LOG2;
// const PADDING: i32 = 2;
pub const VIEWPORT: coords::Rectangle = coords::Rectangle {
    pos: coords::Cube { x: 0, y: 0 },
    width: SIZE as i32,
    height: SIZE as i32,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct TorusTile<CellT: Copy> {
    // I think we don't technically need the Box, just to make sure it's on the heap.
    // Could also use "Vec", but I guess compile-time size will allow additional optimizations?
    // (At least with g++ and Eigen compile-time size was really helping. Maybe test this theory at some point.)
    // data: Box<[CellT; (SIZE * SIZE) as usize]>,  // only works with serde up to 32 elements
    data: Box<[CellT]>,
    // old C++ code, with padding for border-conditions:
    // (note: Padding on each tile is probably not even required for performance.
    //        We could use a huge memory block instead of tiles and do loop tiling,
    //        if we do not want the "inifinite space of tiles" feature.
    //        Though tiling might be good, to keep things on the same memory page?
    //        This is for later. Let's do something that works first.)
    //
    // struct Tile {
    //     CellContent data[(size + 2*padding) * (size + 2*padding)]{};
    //     static constexpr int stride_x = 1;
    //     static constexpr int stride_y = size+2*padding;
    //     static constexpr int stride_z = stride_y - stride_x;
    //     CellContent * data_inside() {
    //     return data + padding*stride_y + padding*stride_x;
    //  }
    //};
}

impl<CellT: Copy> FromIterator<CellT> for TorusTile<CellT> {
    fn from_iter<T: IntoIterator<Item = CellT>>(iter: T) -> Self {
        let data: Box<[CellT]> = iter.into_iter().collect();
        if data.len() != (SIZE * SIZE) as usize {
            panic!("TorusTile created from iterator of wrong size.")
        }
        TorusTile { data }
    }
}

// type NeighbourIterMut<'t> = std::iter::Zip<std::slice::IterMut<'t, CellT>, NeighbourIter<'t>>;

impl<CellT: Copy> TorusTile<CellT> {
    pub fn new(fill: CellT) -> Self {
        TorusTile {
            data: Box::new([fill; (SIZE * SIZE) as usize]),
        }
    }

    pub fn from_fn(mut cb: impl FnMut(coords::Cube) -> CellT) -> Self {
        TorusTile {
            data: Box::<[CellT; (SIZE * SIZE) as usize]>::new(core::array::from_fn(|idx| {
                cb(Self::pos_from_index(idx))
            })),
        }
    }

    // XXX this should take coords::Axial, etc.
    fn index(pos: coords::Cube) -> usize {
        // cube to axial (OPTIMIZE: for iteration we should use axial coordinates to begin with)
        let q = (pos.x as u32) % SIZE;
        let r = (pos.z() as u32) % SIZE;
        (r * SIZE + q) as usize
    }

    // XXX this should be simpler once we use coords::Axial or similar
    fn pos_from_index(idx: usize) -> coords::Cube {
        assert!(idx <= SIZE as usize * SIZE as usize);
        let r = idx as u32 / SIZE;
        let q = idx as u32 % SIZE;
        coords::Cube {
            x: q as i32,
            y: -(q as i32) - (r as i32),
        }
    }

    pub fn random_pos(&self, rng: &mut impl Rng) -> coords::Cube {
        let x = rng.random_range(0..SIZE as i32);
        let y = -x - rng.random_range(0..SIZE as i32); // ugh.
        coords::Cube { x, y }
    }

    pub fn is_same_pos(&self, pos1: coords::Cube, pos2: coords::Cube) -> bool {
        Self::index(pos1) == Self::index(pos2)
    }

    pub fn set_cell(&mut self, pos: coords::Cube, cell: CellT) {
        self.data[Self::index(pos)] = cell;
    }

    pub fn cell(&self, pos: coords::Cube) -> CellT {
        self.data[Self::index(pos)]
    }

    pub fn neighbours(&self, center_pos: coords::Cube) -> [CellT; 6] {
        // const DIR2DELTA: [(i32, i32); 6] = [(1, 0), (0,1), (-1,1), (-1, 0), (0,-1), (1,-1)];
        let neigh = |idx| {
            let dir = Direction::from_int(idx);
            let pos = center_pos + dir;
            self.cell(pos)
        };
        [neigh(0), neigh(1), neigh(2), neigh(3), neigh(4), neigh(5)]
    }

    /// Iterate over all cells (in axial-storage order), yielding the cell and its 6 neighbours
    pub fn iter_radius_1(&self) -> NeighbourIter<CellT> {
        // Note: We might use ::ndarray::ArrayBase::windows() if it wasn't for the wrapping borders.
        NeighbourIter {
            tile: self,
            q: 0,
            r: 0,
        }
    }

    /*
    /// For convolution-like operations
    pub fn mutate_with_radius_1<F>(&mut self, mut f: F)
    where
        // note to self: I think FnMut instead of Fn implies that we should
        // iterate in well-defined order...? For reproducible rng seed at least,
        // order should not be thought of an unstable implementation detail.
        F: FnMut(&mut CellT, NeighbourCells<CellT>),
    {
        // OPTIMIZE: could touch less memory by only keeping a copy of the previous line
        // OPTIMIZE: provide a cheaper method for accessing only two neighbours on the same axis
        let tile_old = self.clone();
        let iter_old_neighbours = tile_old.iter_radius_1();
        let iter_mut_center = self.data.iter_mut();
        for (center, (_center_old, neighbours)) in iter_mut_center.zip(iter_old_neighbours) {
            f(center, neighbours)
        }
    }
    */

    // ??? do we gain something by returning "impl ExactSizeIterator" instead of "impl Iterator"?
    // Probably it is enough that the actual instance type is "impl ExactSizeIterator"...?
    pub fn iter_cells(&self) -> impl ExactSizeIterator<Item = &CellT> {
        self.data.iter()
    }
}

impl<CellT: Default + Copy> Default for TorusTile<CellT> {
    fn default() -> Self {
        TorusTile::new(CellT::default())
    }
}

pub struct NeighbourIter<'t, CellT: Copy> {
    tile: &'t TorusTile<CellT>,
    q: i32,
    r: i32,
}

impl<CellT> Iterator for NeighbourIter<'_, CellT>
where
    CellT: Copy,
{
    type Item = Neighbourhood<CellT>;
    fn next(&mut self) -> Option<Self::Item> {
        // there is probably some rust-ish was to avoid doing this...
        // maybe should use an iterator that just yields the indices, separate from the data access?
        if self.r >= SIZE as i32 {
            return None;
        }
        let (q, r) = (self.q, self.r);
        self.q += 1;
        if self.q >= SIZE as i32 {
            self.r += 1;
            self.q = 0;
        }

        // axial to cube
        let center_pos = coords::Cube { x: q, y: -r - q };
        let neighbours = self.tile.neighbours(center_pos);
        let center = self.tile.cell(center_pos);
        Some(Neighbourhood { center, neighbours })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (SIZE * SIZE) as usize;
        (len, Some(len))
    }
}

impl<CellT: Copy> ExactSizeIterator for NeighbourIter<'_, CellT> {}
