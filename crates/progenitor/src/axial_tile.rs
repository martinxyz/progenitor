use crate::{
    coords::{self, Cube, Direction},
    Neighbourhood,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AxialTile<CellT: Copy> {
    width: i32,
    height: i32,
    // fill: CellT,
    data: Vec<CellT>,
}

impl<CellT: Copy> AxialTile<CellT> {
    fn new_internal(width: i32, height: i32, create: impl FnOnce() -> Vec<CellT>) -> Self {
        assert!(width > 0 && height > 0);
        assert!(width as i64 * height as i64 == (width * height) as i64);
        AxialTile {
            width,
            height,
            data: create(),
        }
    }

    pub fn new(width: i32, height: i32, fill: CellT) -> Self {
        Self::new_internal(width, height, || {
            vec![fill; (width as usize) * (height as usize)]
        })
    }

    pub fn from_fn(width: i32, height: i32, mut cb: impl FnMut(coords::Cube) -> CellT) -> Self {
        let pos_from_index = |idx: i32| -> Cube {
            let r = idx as u32 / width as u32;
            let q = idx as u32 % width as u32;
            coords::Cube {
                x: q as i32,
                y: -(q as i32) - (r as i32),
            }
        };
        Self::new_internal(width, height, || {
            (0..width * height)
                .map(|idx| cb(pos_from_index(idx)))
                .collect()
        })
    }

    fn index(&self, pos: Cube) -> usize {
        // cube to axial (OPTIMIZE: for iteration we should use axial coordinates to begin with)
        let q = pos.x;
        let r = pos.z();
        (r * self.width + q) as usize
    }

    pub fn valid(&self, pos: Cube) -> bool {
        let q = pos.x;
        let r = pos.z();
        (q >= 0 && q < self.width) && (r >= 0 && r < self.height)
    }

    pub fn set_cell(&mut self, pos: Cube, cell: CellT) {
        assert!(self.valid(pos));
        let idx = self.index(pos);
        self.data[idx] = cell;
    }

    pub fn cell(&self, pos: Cube) -> Option<CellT> {
        if self.valid(pos) {
            Some(self.data[self.index(pos)])
        } else {
            None
        }
    }

    pub fn neighbours(&self, center_pos: Cube) -> [(Direction, Option<CellT>); 6] {
        let neigh = |idx| {
            let dir = Direction::from_int(idx);
            let pos = center_pos + dir;
            (dir, self.cell(pos))
        };
        [neigh(0), neigh(1), neigh(2), neigh(3), neigh(4), neigh(5)]
    }

    // ??? do we gain something by returning "impl ExactSizeIterator" instead of "impl Iterator"?
    // Probably it is enough that the actual instance type is "impl ExactSizeIterator"...?
    pub fn iter_cells(&self) -> impl ExactSizeIterator<Item = &CellT> {
        self.data.iter()
    }

    pub fn iter_cells_mut(&mut self) -> impl ExactSizeIterator<Item = &mut CellT> {
        self.data.iter_mut()
    }

    pub fn area(&self) -> i32 {
        self.width * self.height
    }

    pub fn random_pos(&self, rng: &mut impl Rng) -> Cube {
        let x = rng.gen_range(0..self.width);
        let y = -x - rng.gen_range(0..self.height); // ugh.
        Cube { x, y }
    }

    fn neighbourhood_rq(&self, r: i32, q: i32) -> Neighbourhood<CellT> {
        let index = r * self.width + q;
        const OFFSETS: [(i32, i32); 6] = [
            // (r,q)
            (-1, 0),
            (-1, 1),
            (0, 1),
            (1, 0),
            (1, -1),
            (0, -1),
        ];
        Neighbourhood {
            center: self.data[index as usize],
            neighbours: OFFSETS.map(|(dr, dq)| self.data[(index + self.width * dr + dq) as usize]),
        }
    }

    pub fn neighbourhood(&self, pos: Cube) -> Option<Neighbourhood<CellT>> {
        let q = pos.x;
        let r = pos.z();
        let r_outside = r < 1 || r > self.height - 2;
        let q_outside = q < 1 || q > self.width - 2;
        if r_outside || q_outside {
            return None;
        }
        Some(self.neighbourhood_rq(r, q))
    }

    pub fn iter_valid_neighbourhoods(&self) -> impl Iterator<Item = Neighbourhood<CellT>> + '_ {
        (1..self.height - 1)
            .flat_map(move |r| (1..self.width - 1).map(move |q| self.neighbourhood_rq(r, q)))
    }

    /// Cellular automaton step
    pub fn ca_step<Cell2: Copy>(
        &self,
        fill: Cell2,
        mut rule: impl FnMut(Neighbourhood<CellT>) -> Cell2,
    ) -> AxialTile<Cell2> {
        let mut result = AxialTile::new(self.width, self.height, fill);
        for r in 1..self.height - 1 {
            for q in 1..self.width - 1 {
                let index = r * self.width + q;
                result.data[index as usize] = rule(self.neighbourhood_rq(r, q));
            }
        }
        result
    }

    /// Count true/false transitions in the binarized image
    pub fn count_edges(&self, mut predicate: impl FnMut(CellT) -> bool) -> i32 {
        self.iter_valid_neighbourhoods()
            .map(|Neighbourhood { center, neighbours }| {
                let c = predicate(center);
                neighbours[0..3]
                    .iter()
                    .take(3) // only count each edge once
                    .map(|&neighbour| (predicate(neighbour) != c) as i32)
                    .sum::<i32>()
            })
            .sum()
    }

    /// Viewport for implementing HexgridView
    pub fn viewport(&self) -> coords::Rectangle {
        coords::Rectangle {
            pos: Cube { x: 0, y: 0 },
            width: self.width + self.height / 2,
            height: self.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coords::Offset;

    #[test]
    fn test_count_edges() {
        for offset in Direction::all() {
            let mut cells = AxialTile::new(20, 30, 0);
            let center: Cube = Offset { col: 5, row: 5 }.into();
            cells.set_cell(center, 1);
            cells.set_cell(center + offset, 2);

            // no edges
            assert_eq!(cells.count_edges(|_| false), 0);
            assert_eq!(cells.count_edges(|_| true), 0);
            // hex in center to its neighbours (each counted twice)
            assert_eq!(cells.count_edges(|i| i == 1), 6);
            assert_eq!(cells.count_edges(|i| i != 1), 6);
            // two adjacent hexes
            assert_eq!(cells.count_edges(|i| i != 0), 6 + 6 - 2);
        }
    }
}
