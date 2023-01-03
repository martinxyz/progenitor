#![feature(test)]
use progenitor::coords::Cube;
use progenitor::sim1::{Cell, CellType, CellTypeRef, World};
use progenitor::Simulation;

#[test]
fn initialization_should_be_inert() {
    let mut w = World::new();
    let pos = Cube { x: 5, y: 5 };
    assert_eq!(w.cell(pos), Cell::default());
    w.step();
    assert_eq!(w.cell(pos), Cell::default());
}

#[test]
fn simple_self_transformation() {
    let mut w = World::new();
    let empty_cell = CellTypeRef(0);
    let dying_cell = CellTypeRef(1);
    let persistent_cell = CellTypeRef(2);

    w.types[dying_cell] = CellType {
        transform_at_random_p: 128,
        transform_into: empty_cell,
        ..CellType::default()
    };
    let pos0 = Cube { x: 0, y: 0 };
    let pos1 = Cube { x: 5, y: 5 };
    let pos2 = Cube { x: 5, y: 6 };
    w.set_cell(pos1, w.types.create_cell(dying_cell));
    w.set_cell(pos2, w.types.create_cell(persistent_cell));
    assert_eq!(w.cell(pos0).cell_type, empty_cell);
    assert_eq!(w.cell(pos1).cell_type, dying_cell);
    assert_eq!(w.cell(pos2).cell_type, persistent_cell);
    for i in 0..10 {
        w.step();
        assert_eq!(w.cell(pos0), Cell::default(), "at step {}", i);
        assert_eq!(w.cell(pos1), Cell::default(), "at step {}", i);
        assert_eq!(w.cell(pos2).cell_type, persistent_cell, "at step {}", i);
    }
}

#[test]
fn simple_growth() {
    let mut w = World::new();
    let growing_cell: CellTypeRef = CellTypeRef(1);
    w.types[growing_cell] = CellType {
        priority: 1,
        grow_child_type: growing_cell,
        grow_p: 128,
        ..CellType::default()
    };
    let pos1 = Cube { x: 5, y: 5 };
    w.set_cell(pos1, w.types.create_cell(growing_cell));
    let count_growing_cells = |w: &World| {
        w.iter_cells()
            .filter(|c| c.cell_type == growing_cell)
            .count()
    };
    assert_eq!(1, count_growing_cells(&w));
    w.step();
    assert_eq!(1 + 6, count_growing_cells(&w));
    w.step();
    assert_eq!(1 + 6 + 12, count_growing_cells(&w));
}

extern crate test;
use test::Bencher;

#[bench]
fn benchtest(b: &mut Bencher) {
    let mut w = World::new();
    let growing_cell = CellTypeRef(0);
    w.types[growing_cell] = CellType {
        priority: 1,
        grow_child_type: CellTypeRef(1), // self-pointer !!! very bad API
        ..CellType::default()
    };
    let pos1 = Cube { x: 5, y: 5 };
    w.set_cell(pos1, w.types.create_cell(growing_cell));
    /*
    let count_growing_cells = |w: &World| {
        w.iter_cells().filter(|c| c.get_type() == growing_cell).count()
    };
    b.iter(|| count_growing_cells(&w));
    */
    b.iter(|| w.step());
}
