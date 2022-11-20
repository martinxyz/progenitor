use hex2d::Direction;
use serde::Serialize;

use crate::{coords, SIZE};

// (Unfinished architectural brainstorm notes:)
//
// ... technically, a Simulation would consist of two types:
// - A) immutable parameters (e.g. neural net weights, rules)
// - B) mutable state. (...possibly batched?)
//
// Only state needs to be "impl Serialize", for snapshots.
// This would remove ambiguity (Undo a snapshot, or both snapshot and sim parameters?)
// Problem: most methods need access to the simulation params.
//
// ... the Simulation doesn't need to own parameter type, it can just get it passed to tick()...?
// ... so the Simulation would...hm.
// The trait is just an inteface! It cannot own data. It's not a base class.
// But it can declare types. And it needs to be implemented on some type.
// ...
// We could declare an associated type "SimulationState" (default to Simulation?)
// step(): SimulationState -> SimulationState? Nope, too pure.
//
// step(&self, &mut SimulationState) maybe. (self no longer mutable)
//
// Or the other way around: the "Simulation" trait is just for the state.
// It gets passed a SimulationParams everywhere?
// ...
// Or: instead of implementing default methods on the trait here,
//     implement free-standing functions, like:
//     fn get_cells_rectangle(Simulation, mut Simulation::State)
// Surely, the same Simulation can have multiple SimulationState(s) living anywhere,
// and we can also continue a SimulationState with a different Simulation, though
// not within tick()
//
// or re-read this: https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods

// What about:
// trait Simulation {
//    type State: Clone;
//    // fn initialize(&self, seed) -> State;  // <-- this is never used in a generic way; initialization is always special-purpose.
//    fn step(&self, &mut State);
//    fn steps(), sensible_step_size_hint(), ...;
//    fn get_cell_view(&self, &state: State, pos: coords::Cube) -> Option<CellView>;
//    fn get_cell_text(&self, &state: State, pos: coords::Cube) -> Option<String> {
//    // ^ now the map can be part of Simulation (with agents being the State), or part of the State, or both,
//    // and we could now implement a generic SnapshotManager<Simulation>.
// }
// Or, small traits:
// trait InspectableGrid {
//    fn steps
// }

pub trait Simulation {
    fn step(&mut self);
    // Note to self: do not try again to optimize this by returning a trait like
    // "impl Iterator". It may not be fully impossible, but the effort is
    // completely wasted. This API is for rendering; an extra copy or five of
    // this Vec<SmallThing> will be neglegible compared to rendering.
    fn get_cell_view(&self, pos: coords::Cube) -> Option<CellView>;
    fn get_cell_text(&self, pos: coords::Cube) -> Option<String> {
        let cv = self.get_cell_view(pos)?;
        let mut lines = Vec::with_capacity(3);
        lines.push(format!("Type: {}", cv.cell_type));
        if let Some(e) = cv.energy {
            lines.push(format!("Energy: {}", e));
        }
        if let Some(dir) = cv.direction {
            lines.push(format!("Direction: {}", coords::compass_str(dir)));
        }
        Some(lines.join("\n"))
    }

    /// Save the simulation's state. Does not need to include the simulation's
    /// parameters, only enough information to undo a `.step()`.
    fn save_state(&self) -> Vec<u8>;
    fn load_state(&mut self, data: &[u8]);

    fn get_cells_rectangle(&self) -> Vec<CellView> {
        let pos = coords::Cube { x: 0, y: 0 };
        coords::iterate_rectangle(pos, SIZE as i32, SIZE as i32)
            .map(|coord| {
                self.get_cell_view(coord).unwrap_or(CellView {
                    cell_type: 255,
                    ..Default::default()
                })
            })
            .collect()
    }
    // useful when called via trait object (allows loop unrolling)
    fn steps(&mut self, count: usize) {
        for _ in 0..count {
            self.step()
        }
    }
}

#[derive(Default, Serialize, Debug)]
pub struct CellView {
    pub cell_type: u8,
    pub energy: Option<u8>,
    pub direction: Option<Direction>,
}
