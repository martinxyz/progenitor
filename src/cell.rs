use crate::coords::Direction;
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

/// Reference to a `CellType`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellTypeRef(pub u8);

/// State of a cell
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub cell_type: CellTypeRef,
    pub energy: u8,
    pub heading: Direction,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            cell_type: CellTypeRef(0),
            energy: 0,
            heading: Direction::YZ,
        }
    }
}

/// Cell update rules
///
/// Rules how `Cell` states interact
// note: "Copy" required for array initialization
#[derive(Debug, Clone, Copy)]
pub struct CellType {
    pub priority: i8,
    pub initial_energy: u8,
    pub transform_at_random_p: u8, // probability (0 = never, 128 = always)
    pub transform_into: CellTypeRef,
    pub grow_child_type: CellTypeRef,
    pub grow_p: u8, // probability (0 = never, 128 = always) (XXX misnamed for values >128)
}

impl CellType {
    pub fn default() -> CellType {
        CellType {
            priority: 0,
            initial_energy: 0,
            transform_at_random_p: 0,
            transform_into: CellTypeRef(0),
            grow_child_type: CellTypeRef(0),
            grow_p: 0,
        }
    }
}

pub const MAX_CELL_TYPES: usize = 256;
pub struct CellTypes {
    types: Box<[CellType; MAX_CELL_TYPES]>,
}

impl Index<CellTypeRef> for CellTypes {
    type Output = CellType;
    fn index(&self, index: CellTypeRef) -> &CellType {
        &self.types[index.0 as usize]
    }
}

impl IndexMut<CellTypeRef> for CellTypes {
    fn index_mut(&mut self, index: CellTypeRef) -> &mut CellType {
        &mut self.types[index.0 as usize]
    }
}

impl CellTypes {
    pub fn new() -> Self {
        let empty_type = CellType::default();
        CellTypes {
            types: Box::new([empty_type; 256]),
        }
    }

    pub fn create_cell(&self, cell_type: CellTypeRef) -> Cell {
        let ct = self[cell_type];
        Cell {
            cell_type,
            energy: ct.initial_energy,
            ..Cell::default()
        }
    }
}

impl Default for CellTypes {
    fn default() -> Self {
        CellTypes::new()
    }
}
