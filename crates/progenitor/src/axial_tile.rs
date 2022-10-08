use crate::coords::Cube;
use hex2d::Direction;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AxialTile<CellT: Copy> {
    width: i32,
    height: i32,
    // fill: CellT,
    data: Vec<CellT>,
}

impl<CellT: Copy> AxialTile<CellT> {
    pub fn new(width: i32, height: i32, fill: CellT) -> Self {
        let width = width;
        let height = height;
        let data = vec![fill; (width as usize) * (height as usize)];
        AxialTile {
            width,
            height,
            // fill,
            data,
        }
    }

    fn index(&self, pos: Cube) -> usize {
        // cube to axial (OPTIMIZE: for iteration we should use axial coordinates to begin with)
        let q = pos.x as i32;
        let r = pos.z() as i32;
        (r * self.width + q) as usize
    }

    pub fn valid(&self, pos: Cube) -> bool {
        let q = pos.x as i32;
        let r = pos.z() as i32;
        (q >= 0 && q < self.width) && (r >= 0 && r < self.height)
    }

    pub fn set_cell(&mut self, pos: Cube, cell: CellT) {
        assert!(self.valid(pos));
        let idx = self.index(pos);
        self.data[idx] = cell;
    }

    pub fn get_cell(&self, pos: Cube) -> Option<CellT> {
        if self.valid(pos) {
            Some(self.data[self.index(pos)])
        } else {
            None
        }
    }

    pub fn get_neighbours(&self, center_pos: Cube) -> [(Direction, Option<CellT>); 6] {
        // const DIR2DELTA: [(i32, i32); 6] = [(1, 0), (0,1), (-1,1), (-1, 0), (0,-1), (1,-1)];
        let neigh = |idx| {
            let dir = Direction::from_int(idx);
            let pos = center_pos + dir;
            (dir, self.get_cell(pos))
        };
        [neigh(0), neigh(1), neigh(2), neigh(3), neigh(4), neigh(5)]
    }

    /*
    /// Iterate over all cells (in axial-storage order), yielding the cell and its 6 neighbours
    pub fn iter_radius_1(&self) -> NeighbourIter<CellT> {
        // Note: We might use ::ndarray::ArrayBase::windows() if it wasn't for the wrapping borders.
        NeighbourIter {
            tile: self,
            pos: Cube { x: 1, y: 1 },
        }
    }
     */

    // ??? do we gain something by returning "impl ExactSizeIterator" instead of "impl Iterator"?
    // Probably it is enough that the actual instance type is "impl ExactSizeIterator"...?
    pub fn iter_cells(&self) -> impl ExactSizeIterator<Item = &CellT> {
        self.data.iter()
    }

    pub fn area(&self) -> i32 {
        self.width * self.height
    }
}
