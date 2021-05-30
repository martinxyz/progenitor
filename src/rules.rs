use crate::{
    cell::CellTypes,
    coords::{Direction, DirectionSet},
    Cell, CellTypeRef,
};
use rand::{seq::SliceRandom, Rng};

// Temporary state of cell (intermediate calculation)
#[derive(Debug, Clone, Copy)]
pub struct CellTemp {
    pub cell: Cell,
    grow_celltype: CellTypeRef,
    grow_directions: DirectionSet,
    grow_prio: i8,
}

pub fn prepare_step(types: &CellTypes, rng: &mut impl Rng, cur: Cell) -> CellTemp {
    let ct = types[cur.cell_type];
    let growth = match ct.grow_p {
        0 => DirectionSet::none(),
        prob if prob < 128 => DirectionSet::matching(|_| rng.gen_range(0..128) < prob),
        128 => DirectionSet::all(),
        129 => DirectionSet::single(cur.heading),
        // Also allow a single random direction? But in a better way...
        _ => DirectionSet::single(*Direction::all().choose(rng).unwrap()),
    };
    CellTemp {
        cell: cur,
        grow_celltype: ct.grow_child_type,
        grow_directions: growth,
        grow_prio: types[ct.grow_child_type].priority,
    }
}

pub fn execute_step(
    types: &CellTypes,
    rng: &mut impl Rng,
    cur: Cell,
    neighbours: [(Direction, CellTemp); 6],
) -> Cell {
    let growth_result: Option<Cell> = {
        let base_prio = types[cur.cell_type].priority;
        let candidates = neighbours
            .iter()
            .filter(|&(dir, temp)| {
                temp.grow_directions.contains(-*dir)
                    && temp.grow_prio > base_prio
                    && temp.grow_prio >= types[temp.grow_celltype].priority
            })
            .collect::<Vec<_>>();
        candidates.choose(rng).and_then(|&(dir, temp)| {
            let ct = temp.grow_celltype;
            if ct == cur.cell_type {
                None
            } else {
                let mut cell = types.create_cell(ct);
                cell.heading = -*dir;
                Some(cell)
            }
        })
    };
    let tick_result: Cell = {
        // hm. Nothing else to do yet? No counters to tick? no energy to absorb? no cells to swap?
        self_transform(types, rng, cur)
    };
    growth_result.unwrap_or(tick_result)
}

fn self_transform(types: &CellTypes, rng: &mut impl Rng, cur: Cell) -> Cell {
    let ct = types[cur.cell_type];
    // let trigger1 = if let Some(energy) = ct.transform_at_energy {
    //     cur.energy == energy
    // } else {
    //     false
    // };
    let trigger1 = false;
    let trigger2 = match ct.transform_at_random_p {
        0 => false,
        prob if prob < 128 => rng.gen_range(0..128) < prob,
        _ => true,
    };
    if trigger1 || trigger2 {
        Cell {
            heading: cur.heading,
            ..types.create_cell(ct.transform_into)
        }
    } else {
        cur
    }
}
