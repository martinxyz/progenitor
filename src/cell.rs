use crate::coords::{Direction, DirectionSet};
use rand::Rng;
use rand::seq::SliceRandom;
use std::ops::{Index, IndexMut};

/// Reference to a `CellType`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellTypeRef(pub u8);

/// State of a cell
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub cell_type: CellTypeRef,
    pub value1: u8, // less pub please, representation should probably be internal
    pub value2: u8,
    pub particle: bool,
    temp: CellTemp,
}

// Temporary state of cell during transaction resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CellTemp {
    pub transact: DirectionSet,
}

impl Cell {
    pub fn empty() -> Cell {
        Cell {
            cell_type: CellTypeRef(0),
            value1: 0,
            value2: 0,
            particle: false,
            temp: Default::default(),
        }
    }
}

/*
#[derive(Debug, Clone, Copy)]
pub enum ValueTransfer {
    Set(u8),
    Copy,
    Increment,
    // Randomize, ...
}

impl ValueTransfer {
    fn transfer(self, value: u8) -> u8 {
        match self {
            Self::Set(v) => v,
            Self::Copy => value,
            Self::Increment => {
                if value < 255 {
                    value + 1
                } else {
                    value
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ValueSpec {
    pub initial: u8,
    // pub tick: ValueTransfer,
    // pub transfer_child: ValueTransfer,
    // pub transfer_self: ValueTransfer,
}

pub enum TransformTrigger {
    Never,
    Value1Equals(u8),
}
*/

/// Cell update rules
///
/// Rules how `Cell` states interact
// note: "Copy" required for array initialization
#[derive(Debug, Clone, Copy)]
pub struct CellType {
    // pub value1_spec: ValueSpec,
    pub transform_at_value1: Option<u8>,
    pub transform_into: CellTypeRef,
    pub max_children: u8,
    pub child_type: CellTypeRef,
    pub skip_transaction_p: u8, // probability (0 = never, 128 = always)
    pub child_at_parent_location_p: u8, // probability (0 = never, 128 = always)
    pub priority: i8,           // 0 = "default" (replacing a cell requires higher priority)
}

impl CellType {
    pub fn default() -> CellType {
        CellType {
            /*
            value1_spec: ValueSpec {
                initial: 0,
                // tick: ValueTransfer::Copy,
                // transfer: ValueTransfer::Set(0),
            },
            */
            transform_at_value1: None,
            transform_into: CellTypeRef(0),
            max_children: 0,
            child_type: CellTypeRef(0),
            skip_transaction_p: 0,
            child_at_parent_location_p: 0,
            priority: 0,
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
        Cell {
            cell_type,
            value1: 0,
            value2: 0,
            particle: false,
            temp: CellTemp {
                // New cells created during a transaction shall not create a
                // transactions themselves during the same tick().
                transact: DirectionSet::none()
            }
        }
        // ...more fancy initialization might be configurable in CellType in the future.
    }

    pub fn prepare_transaction(&self, rng: &mut impl Rng, cur: Cell) -> Cell {
        let ct = self[cur.cell_type];
        let transact = match ct.skip_transaction_p {
            0 => DirectionSet::all(),
            128 => DirectionSet::none(),
            prob if prob < 128 => DirectionSet::matching(|_| {
                rng.gen_range(0, 128) >= prob
            }),
            // Also allow a single random direction? But in a better way...
            _ => DirectionSet::single(*Direction::all().choose(rng).unwrap()),
        };
        // let dir = Direction::from_int(rng.gen_range(0, 6));
        Cell {
            // allow all 6 directions in the same tick():
            temp: CellTemp { transact },
            ..cur
        }
    }

    pub fn clear_transaction(&self, cur: Cell) -> Cell {
        Cell {
            temp: Default::default(),
            ..cur
        }
    }

    pub fn get_transaction(&self, cur: Cell, next: Cell, dir: Direction) -> Transaction {
        let cur_ct = self[cur.cell_type];
        let next_ct = self[next.cell_type];

        if cur.temp.transact.contains(dir) {
            if cur.value1 < cur_ct.max_children && cur_ct.priority > next_ct.priority {
                let res = Transaction {
                    // Note that a SplitTransactions should only be created if:
                    // 1. next_cell has higher priority than the cell it replaces, and
                    // 2. cur_cell does not have higher priority than the cell it replaces.
                    //
                    // Those rules ensure that, even if there is a conflict,
                    // transactions can only replace a cell if the creator of the
                    // transaction has higher priority than the cell being replaced.
                    split: SplitTransaction::Split {
                        cur_cell: Cell {
                            value1: cur.value1 + 1,
                            ..cur
                        },
                        next_cell: self.create_cell(cur_ct.child_type),
                    },
                };
                return res;
            }
        }
        // if (next.cell_type == 0 &&
        //     cur.cell_type != 0 &&
        //     cur.child_count < cur_ct.child_maxcount) {
        //   CellContent child1 {
        //     .cell_type = cur.cell_type,
        //     .child_count = static_cast<uint8_t>(std::min(cur.child_count + 1, 255)),
        //   };
        //   CellContent child2 {
        //     .cell_type = cur_ct.child,
        //   };
        //   bool child1_first = probability(cur_ct.child_at_parent_location_p);
        //   return {
        //     .split = true,
        //     .child1 = child1_first ? child1 : child2,
        //     .child2 = child1_first ? child2 : child1,
        //   };
        // }

        Transaction {
            split: SplitTransaction::None,
        }
    }

    pub fn execute_transactions(
        &self,
        prev_to_cur: Transaction,
        cur: Cell,
        cur_to_next: Transaction,
    ) -> Cell {
        if let SplitTransaction::Split {
            cur_cell: _,
            next_cell,
        } = prev_to_cur.split
        {
            // note: prev_to_cur takes priority (cell priorities are checked again)
            next_cell
        } else if let SplitTransaction::Split {
            cur_cell,
            next_cell: _,
        } = cur_to_next.split
        {
            cur_cell
        } else {
            cur
        }
    }
}

impl Default for CellTypes {
    fn default() -> Self {
        CellTypes::new()
    }
}

#[derive(Debug)]
pub struct Transaction {
    split: SplitTransaction,
    // enum Transaction
    // Noop,
    // Death,
    // Grow(i32),
    // Child {
    //     cell_type: u8,
    //     child_count: u8,
    // },
    // Swap
}

#[derive(Debug, PartialEq)]
enum SplitTransaction {
    None,
    Split { cur_cell: Cell, next_cell: Cell },
}
