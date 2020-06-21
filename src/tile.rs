use crate::cell;
use cell::Cell;
use hex2d::{Coordinate, Direction};

const SIZE_LOG2: u32 = 3;
pub const SIZE: u32 = 1 << SIZE_LOG2;
// const PADDING: i32 = 2;


// This doesn't exist in hex2d?! Or I didn't find it?
pub fn offset_to_cube(col: i32, row: i32) -> Coordinate {
    // taken from redblob - XXX this may need unit-tests (does it work with with negative pos?) if I'm going to keep this
    let x = col - (row - (row&1)) / 2;
    let z = row;
    let y = -x-z;
    Coordinate::new(x, y)
}

#[derive(Clone)]
pub struct Tile {
    // I think we don't technically need the Box, just to make sure it's on the heap.
    // Could also use "Vec", but I guess compile-time size will allow additional optimizations?
    // (At least with g++ and Eigen compile-time size was really helping. Maybe test this theory at some point.)
    data: Box<[Cell; (SIZE*SIZE) as usize]>,

    // old C++ code, with padding for border-conditions:
    // (note: Padding on each tile is probably not even required for performance.
    //        We could use a huge memory block instead of tiles and do loop tiling,
    //        if we do not want the "inifinite space of tiles" feature.
    //        Though it tiling might be good, to keep things on the same memory page?
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

// type NeighbourIterMut<'t> = std::iter::Zip<std::slice::IterMut<'t, Cell>, NeighbourIter<'t>>;

impl Tile {
    pub fn new() -> Tile {
        let cell = Cell::empty();
        Tile {
            data: Box::new([cell; (SIZE*SIZE) as usize])
        }
    }

    fn get_index(pos: Coordinate) -> usize {
        // cube to axial
        let q = (pos.x as u32) % SIZE;
        let r = (pos.z() as u32) % SIZE;  // this is what I wanted, but... maybe consider wanting something else now?
        (r * SIZE + q) as usize
    }

    pub fn set_cell(&mut self, pos: Coordinate, cell: Cell) {
        self.data[Self::get_index(pos)] = cell;
    }

    pub fn get_cell(&self, pos: Coordinate) -> Cell {
        self.data[Self::get_index(pos)]
    }

    /// Iterator over a rectangle in offset coordinates.
    pub fn iterate_rectangle(pos: Coordinate, width: i32, height: i32) -> impl Iterator<Item = Coordinate> {
        (0..height).map(move |row| (0..width).map(move |col| 
            pos + offset_to_cube(col, row)
        )).flatten()
    }

    /// Iterate over all cells (in unspecified order), yielding the cell and its 6 neighbours
    pub fn iter_radius_1(&self) -> NeighbourIter {
        // Note: We might use ::ndarray::ArrayBase::windows() if it wasn't for the wrapping borders.
        NeighbourIter{tile: &self, q: 0, r: 0}
    }

    /// For convolution-like operations
    pub fn mutate_with_radius_1<F>(&mut self, f: F)
    where F: Fn(&mut Cell, NeighbourCells) {
        // note: could be optimized require only a copy of the previous line
        let tile_old = self.clone();
        let iter_old_neighbours = tile_old.iter_radius_1();
        let iter_mut_center = self.data.iter_mut();
        for (center, (_center_old, neighbours)) in iter_mut_center.zip(iter_old_neighbours) {
            f(center, neighbours)
        }
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = &Cell> {
        self.data.iter()
    }
}

pub struct NeighbourIter<'t> {
    tile: &'t Tile,
    q: i32,
    r: i32,
}

pub type NeighbourCells = [Cell; 6];

impl<'t> Iterator for NeighbourIter<'t> {
    type Item = (Cell, NeighbourCells);
    fn next(&mut self) -> Option<Self::Item> {
        // there is probably some rust-ish was to avoid doing this...
        // maybe should use an iterator that just yields the indices, separate from the data access?
        if self.r >= SIZE as i32 {
            return None
        }
        let (q, r) = (self.q, self.r);
        self.q += 1;
        if self.q >= SIZE as i32 {
            self.r += 1;
            self.q = 0;
        }

        // axial to cube
        let x = q;
        let z = r;
        let y = -z -x;
        let center_pos = Coordinate::new(x, y);
        let mut neighbours = [Cell::empty(); 6];
        for i in 0..6 {
            // const DIR2DELTA: [(i32, i32); 6] = [(1, 0), (0,1), (-1,1), (-1, 0), (0,-1), (1,-1)];
            // for direction in 0..6 {
            //     let (dx, dy) = DIR2DELTA[direction];
            //     neighbours[direction] = self.tile.get_cell(x + dx, y + dy);
            // }
            let pos = center_pos + Direction::from_int(i as i32);
            neighbours[i] = self.tile.get_cell(pos);
        }
        let center = self.tile.get_cell(center_pos);
        Some((center, neighbours))
    }
}
