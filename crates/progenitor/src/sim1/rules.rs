use crate::coords::{Direction, DirectionSet};

use super::cell::{Cell, CellTypeRef, CellTypes, GrowDirection};
use rand::{seq::IteratorRandom, seq::SliceRandom, Rng};

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
        prob if prob < 128 => match ct.grow_dir {
            GrowDirection::All => DirectionSet::matching(|_| rng.gen_range(0..128) < prob),
            GrowDirection::Forward => match rng.gen_range(0..128) < prob {
                false => DirectionSet::none(),
                true => DirectionSet::single(cur.heading),
            },
            GrowDirection::RandomChoice => match rng.gen_range(0..128) < prob {
                false => DirectionSet::none(),
                true => DirectionSet::single(*Direction::all().choose(rng).unwrap()),
            },
        },
        128 => match ct.grow_dir {
            GrowDirection::All => DirectionSet::all(),
            GrowDirection::Forward => DirectionSet::single(cur.heading),
            GrowDirection::RandomChoice => {
                DirectionSet::single(*Direction::all().choose(rng).unwrap())
            }
        },
        _ => panic!("growth probability out of range"),
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
    // move particles
    let mut cur = cur;
    cur.particles = neighbours
        .iter()
        .fold(DirectionSet::none(), |ds, &(dir, temp)| {
            ds.with(-dir, temp.cell.particles.contains(-dir))
        });
    let cur = cur;

    let growth_result: Option<Cell> = neighbours
        .iter()
        .filter(|(dir, temp)| {
            let base_prio = types[cur.cell_type].priority;
            temp.grow_directions.contains(-*dir)
                && temp.grow_prio > base_prio
                && temp.grow_prio >= types[temp.grow_celltype].priority
        })
        .filter(|(_, temp)| temp.grow_celltype != cur.cell_type)
        .choose(rng)
        .map(|(dir, temp)| {
            let mut cell = types.create_cell(temp.grow_celltype);
            cell.heading = -*dir;
            cell.particles = DirectionSet::all(); // should depend on celltype?
            cell
        });
    let step_result: Cell = {
        // hm. Nothing else to do yet? No counters to tick? no energy to absorb? no cells to swap?
        self_transform(types, rng, cur)
    };
    let next = growth_result.unwrap_or(step_result);
    Cell {
        energy: next.particles.count(),
        particles: next.particles,
        // particles: if next.cell_type == CellTypeRef(0) {
        //     next.particles
        // } else {
        //     next.particles.mirrored()
        // },
        ..next
    }
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
