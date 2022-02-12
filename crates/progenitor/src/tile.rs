use crate::coords;
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;

use coords::Direction;

const SIZE_LOG2: u32 = 5;
/// Tile storage is a SIZE x SIZE parallelogram.
pub const SIZE: u32 = 1 << SIZE_LOG2;
// const PADDING: i32 = 2;

#[derive(Clone, Serialize, Deserialize)]
pub struct Tile<CellT: Copy> {
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

impl<CellT: Copy> FromIterator<CellT> for Tile<CellT> {
    fn from_iter<T: IntoIterator<Item = CellT>>(iter: T) -> Self {
        let data: Box<[CellT]> = iter.into_iter().collect();
        if data.len() != (SIZE * SIZE) as usize {
            panic!("Tile created from iterator of wrong size.")
        }
        Tile { data }
    }
}

// type NeighbourIterMut<'t> = std::iter::Zip<std::slice::IterMut<'t, CellT>, NeighbourIter<'t>>;

impl<CellT: Copy> Tile<CellT> {
    pub fn new(fill: CellT) -> Self {
        Tile {
            data: Box::new([fill; (SIZE * SIZE) as usize]),
        }
    }

    // XXX this should take coords::Axial, etc.
    fn get_index(pos: coords::Cube) -> usize {
        // cube to axial (OPTIMIZE: for iteration we should use axial coordinates to begin with)
        let q = (pos.x as u32) % SIZE;
        let r = (pos.z() as u32) % SIZE;
        (r * SIZE + q) as usize
    }

    pub fn set_cell(&mut self, pos: coords::Cube, cell: CellT) {
        self.data[Self::get_index(pos)] = cell;
    }

    pub fn get_cell(&self, pos: coords::Cube) -> CellT {
        self.data[Self::get_index(pos)]
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

    /// For convolution-like operations
    /*
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

/// Iterator over a rectangle in offset coordinates.
pub fn iterate_rectangle(
    pos: coords::Cube,
    width: i32,
    height: i32,
) -> impl Iterator<Item = coords::Cube> {
    (0..height).flat_map(move |row| (0..width).map(move |col| pos + coords::Offset { col, row }))
}

pub struct NeighbourIter<'t, CellT: Copy> {
    tile: &'t Tile<CellT>,
    q: i32,
    r: i32,
}

pub type NeighbourCells<CellT> = [(Direction, CellT); 6];

impl<CellT> Iterator for NeighbourIter<'_, CellT>
where
    CellT: Copy,
{
    type Item = (CellT, NeighbourCells<CellT>);
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

        // const DIR2DELTA: [(i32, i32); 6] = [(1, 0), (0,1), (-1,1), (-1, 0), (0,-1), (1,-1)];
        let neigh = |idx| {
            let dir = Direction::from_int(idx);
            let pos = center_pos + dir;
            (dir, self.tile.get_cell(pos))
        };
        let neighbours = [neigh(0), neigh(1), neigh(2), neigh(3), neigh(4), neigh(5)];
        let center = self.tile.get_cell(center_pos);
        Some((center, neighbours))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (SIZE * SIZE) as usize;
        (len, Some(len))
    }
}

impl<CellT: Copy> ExactSizeIterator for NeighbourIter<'_, CellT> {}
