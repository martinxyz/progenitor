/// Reference to a registered `CellType`
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellTypeRef(pub u8); // really... pub pub? Should not expose both CellTypeRef internals and CellType.

/// State of a cell
///
/// This includes a reference to the `CellType` and other per-hex information
/// (e.g. counters for individual cells).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    cell_type: CellTypeRef,
    pub value1: u8, // less pub please, representation should probably be internal
    pub value2: u8,
    particle: bool,
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

    pub fn get_particle(&self) -> bool {
        self.particle
    }
    pub fn set_particle(&self, state: bool) -> Self {
        Self {
            particle: state,
            ..*self
        }
    }

    // hm... ugly do-nothing getter, just to have a stable public API? really?
    pub fn get_type(&self) -> CellTypeRef {
        self.cell_type
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
#[derive(Debug, Clone)]
pub struct CellType {
    // pub value1_spec: ValueSpec,
    // pub transform_type: CellTypeRef,
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

/*
impl CellType {
// problem... this one should be in the global list of cell types, not here,
// so it can reference itself by index.
}
*/

pub struct CellTypes {
    // const MAX_CELL_TYPES: usize = 127;  // maybe use fixe-size array instead of Vec?
    // maybe add a name string? (separated from the CellType memory used for calculations)
    types: Vec<CellType>,
}

impl CellTypes {
    pub fn new() -> CellTypes {
        let empty_type = CellType::default();
        // let border = CellType {  ...
        CellTypes {
            types: vec![empty_type],
        }
    }

    // not needed?
    pub fn type_from_cell(&self, cell: Cell) -> &CellType {
        &self.types[cell.cell_type.0 as usize]
    }

    pub fn type_from_typeref(&self, tr: CellTypeRef) -> &CellType {
        &self.types[tr.0 as usize]
    }

    pub fn create_cell(&self, cell_type: CellTypeRef) -> Cell {
        Cell {
            cell_type,
            ..Cell::empty()
        }
        // ...more fancy initialization might be configurable in CellType in the future.
    }

    pub fn add_type(&mut self, cell_type: &CellType) -> CellTypeRef {
        if self.types.len() > 255 {
            panic!("currently at most 256 cell types are supported")
        }
        let cell_type_ref = CellTypeRef(self.types.len() as u8);
        self.types.push(cell_type.clone());
        cell_type_ref
    }

    pub fn get_transaction(&self, cur: Cell, next: Cell) -> Transaction {
        let cur_ct = self.type_from_cell(cur);
        let next_ct = self.type_from_cell(next);
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
