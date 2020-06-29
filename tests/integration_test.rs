#![feature(test)]
extern crate progenitor;
use progenitor::coords::{Cube, Direction};
use progenitor::{Cell, CellType, CellTypeRef, World};

#[test]
fn initialization_should_be_inert() {
    let mut w = World::new();
    let pos = Cube { x: 5, y: 5 };
    assert_eq!(w.get_cell(pos), Cell::empty());
    w.tick(Direction::YZ);
    assert_eq!(w.get_cell(pos), Cell::empty());
}

#[test]
fn simple_self_transformation() {
    let mut w = World::new();
    let dying_cell = w.types.add_type(&CellType {
        transform_at_value1: Some(0),
        transform_into: CellTypeRef(0),
        ..CellType::default()
    });
    let persistent_cell = w.types.add_type(&CellType::default());
    let pos1 = Cube { x: 5, y: 5 };
    let pos2 = Cube { x: 5, y: 6 };
    w.set_cell(pos1, w.types.create_cell(dying_cell));
    w.set_cell(pos2, w.types.create_cell(persistent_cell));
    assert_eq!(w.get_cell(pos1).get_type(), dying_cell);
    assert_eq!(w.get_cell(pos2).get_type(), persistent_cell);
    w.tick(Direction::YZ);
    assert_eq!(w.get_cell(pos1), Cell::empty());
    assert_eq!(w.get_cell(pos2).get_type(), persistent_cell);
}

#[test]
fn simple_growth() {
    let mut w = World::new();
    let growing_cell = w.types.add_type(&CellType {
        air_like: false,
        child_type: CellTypeRef(1), // self-pointer !!! very bad API
        ..CellType::default()
    });
    let pos1 = Cube { x: 5, y: 5 };
    w.set_cell(pos1, w.types.create_cell(growing_cell));
    let count_growing_cells = |w: &World| {
        w.iter_cells()
            .filter(|c| c.get_type() == growing_cell)
            .count()
    };
    assert_eq!(1, count_growing_cells(&w));
    w.tick(Direction::YZ);
    assert_eq!(2, count_growing_cells(&w));
    w.tick(Direction::YZ);
    assert_eq!(3, count_growing_cells(&w));

    w.tick(Direction::XZ);
    assert_eq!(6, count_growing_cells(&w));
}

extern crate test;
use test::Bencher;

#[bench]
fn benchtest(b: &mut Bencher) {
    let mut w = World::new();
    let growing_cell = w.types.add_type(&CellType {
        air_like: false,
        child_type: CellTypeRef(1), // self-pointer !!! very bad API
        ..CellType::default()
    });
    let pos1 = Cube { x: 5, y: 5 };
    w.set_cell(pos1, w.types.create_cell(growing_cell));
    /*
    let count_growing_cells = |w: &World| {
        w.iter_cells().filter(|c| c.get_type() == growing_cell).count()
    };
    b.iter(|| count_growing_cells(&w));
    */
    b.iter(|| w.tick(Direction::YZ));
}
