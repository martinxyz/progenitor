use crate::coords::{Direction, DirectionSet};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

/// Reference to a `CellType`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellTypeRef(pub u8);

/// State of a cell
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub cell_type: CellTypeRef,
    pub energy: u8,         // meaning depends on cell_type
    pub heading: Direction, // generic value2 byte, perhaps? 3bits for heading at least...
}

// Temporary state of cell (intermediate calculation)
#[derive(Copy, Clone, Debug)]
pub struct CellTemp {
    grow_celltype: CellTypeRef,
    grow_directions: DirectionSet,
    grow_prio: i8,
}

// Temporary state of cell (intermediate calculation)
#[derive(Copy, Clone, Debug)]
pub struct EnergyTransfer {
    pub allow_out: DirectionSet, // pub?
    pub allow_in: DirectionSet,
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

    pub fn self_transform(&self, rng: &mut impl Rng, cur: Cell) -> Cell {
        let ct = self[cur.cell_type];
        // let trigger1 = if let Some(energy) = ct.transform_at_energy {
        //     cur.energy == energy
        // } else {
        //     false
        // };
        let trigger1 = false;
        let trigger2 = match ct.transform_at_random_p {
            0 => false,
            prob if prob < 128 => rng.gen_range(0, 128) < prob,
            _ => true,
        };
        if trigger1 || trigger2 {
            self.create_cell(ct.transform_into)
        } else {
            cur
        }
    }

    pub fn prepare_growth(&self, rng: &mut impl Rng, cur: Cell) -> CellTemp {
        let ct = self[cur.cell_type];
        let growth = match ct.grow_p {
            0 => DirectionSet::none(),
            prob if prob < 128 => DirectionSet::matching(|_| rng.gen_range(0, 128) < prob),
            128 => DirectionSet::all(),
            129 => DirectionSet::single(cur.heading),
            // Also allow a single random direction? But in a better way...
            _ => DirectionSet::single(*Direction::all().choose(rng).unwrap()),
        };
        CellTemp {
            grow_celltype: ct.grow_child_type,
            grow_directions: growth,
            grow_prio: self[ct.grow_child_type].priority,
        }
    }

    pub fn execute_growth(
        &self,
        rng: &mut impl Rng,
        cur: Cell,
        neighbours: [(Direction, CellTemp); 6],
    ) -> Cell {
        let base_prio = self[cur.cell_type].priority;
        let candidates = neighbours
            .iter()
            .filter(|&(dir, temp)| {
                temp.grow_directions.contains(-*dir)
                    && temp.grow_prio > base_prio
                    && temp.grow_prio >= self[temp.grow_celltype].priority
            })
            .collect::<Vec<_>>();
        if let Some(&(dir, temp)) = candidates.choose(rng) {
            let ct = temp.grow_celltype;
            if ct == cur.cell_type {
                cur
            } else {
                let mut cell = self.create_cell(ct);
                cell.heading = -*dir;
                cell
            }
        } else {
            cur
        }
    }

    pub fn wants_energy_transfer(&self, cur: Cell, next: Cell, _dir: Direction) -> bool {
        let cur_ct = self[cur.cell_type];
        // let next_ct = self[next.cell_type];
        if cur.energy > 1 {
            // those should be configurable by celltype
            if next.cell_type == cur.cell_type {
                true
            } else {
                cur_ct.grow_p != 0 && next.cell_type == cur_ct.grow_child_type
            }
        } else {
            false
        }
    }
}

impl Default for CellTypes {
    fn default() -> Self {
        CellTypes::new()
    }
}
