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
}

impl Cell {
    pub fn empty() -> Cell {
        Cell {
            cell_type: CellTypeRef(0),
            value1: 0,
            value2: 0,
            particle: false,
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
    pub child_type: CellTypeRef,
    pub skip_transaction_p: u8, // probability (0 = never, 128 = always)
    pub child_at_parent_location_p: u8, // probability (0 = never, 128 = always)
    pub air_like: bool,
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
            child_type: CellTypeRef(0),
            skip_transaction_p: 0,
            child_at_parent_location_p: 0,
            air_like: true,
        }
    }
}

pub const MAX_CELL_TYPES: usize = 256;
pub struct CellTypes(Box<[CellType; MAX_CELL_TYPES]>);

impl Index<CellTypeRef> for CellTypes {
    type Output = CellType;
    fn index(&self, index: CellTypeRef) -> &CellType {
        &self.0[index.0 as usize]
    }
}

impl IndexMut<CellTypeRef> for CellTypes {
    fn index_mut(&mut self, index: CellTypeRef) -> &mut CellType {
        &mut self.0[index.0 as usize]
    }
}

impl CellTypes {
    pub fn new() -> Self {
        let empty_type = CellType::default();
        CellTypes(Box::new([empty_type; 256]))
    }

    pub fn create_cell(&self, cell_type: CellTypeRef) -> Cell {
        Cell {
            cell_type,
            ..Cell::empty()
        }
        // ...more fancy initialization might be configurable in CellType in the future.
    }

    pub fn get_transaction(&self, cur: Cell, next: Cell) -> Transaction {
        let cur_ct = self[cur.cell_type];
        let next_ct = self[next.cell_type];
        // if (probability(cur_ct.skip_transaction_p)) { return {}; }

        if next_ct.air_like {
            // let child_ct = self.type_from_typeref(cur_ct.child_type);
            if !cur_ct.air_like {
                println!("cur: {:?}", cur);
                println!("next: {:?}", next);
                // todo: value1 transfer, max child cound, etc.
                let res = Transaction {
                    split: SplitTransaction::Split {
                        child1: cur,
                        child2: self.create_cell(cur_ct.child_type),
                    },
                };
                println!("res: {:?}", res);
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
        // prev creates child2
        if let SplitTransaction::Split { child1: _, child2 } = prev_to_cur.split {
            assert_eq!(cur_to_next.split, SplitTransaction::None);
            return child2;
        }

        // cur creates child1
        if let SplitTransaction::Split { child1, child2: _ } = prev_to_cur.split {
            assert_eq!(cur_to_next.split, SplitTransaction::None);
            return child1;
        }

        // noop
        cur
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
    Split { child1: Cell, child2: Cell },
}
