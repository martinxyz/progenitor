//! Cellular Automata Helpers

use rand::seq::IteratorRandom;

use crate::coords::Direction;
use crate::{AxialTile, Neighbourhood, SimRng, TorusTile};

/// A cellular automaton with transactions
///
/// This trait implements an CA that can move or swap cells. This is implemented
/// via exclusive transactions between two neighbours. Each cell may request a
/// transaction towards each of its neighbours. A cell will participate at most
/// in one transaction (choosen at random).
///
/// After transactions are done, a normal CA update step follows. Each cell may
/// update itself based on its own state and the state of its neighbours.
///
/// (Technically, the implementation is a stochastic cellular automaton with
/// neighbourhood radius three. The probability for a specific transaction to
/// execute is always greater than 4%. It's 100% in the absence of conflicts.)
pub trait TransactionalCaRule {
    type Cell: Copy;

    /// Checks whether a transaction from source to target is allowed, and what
    /// the result would be. Must be a pure function (no randomness).
    fn transaction(
        &self,
        source: Self::Cell,
        target: Self::Cell,
        direction: Direction,
    ) -> Option<TransactionResult<Self::Cell>>;

    /// Calculates a CA update
    fn step(&self, neighbourhood: Neighbourhood<Self::Cell>, rng: &mut SimRng) -> Self::Cell;
}

#[derive(Clone, Copy)]
pub struct TransactionResult<Cell: Copy> {
    pub source: Cell,
    pub target: Cell,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Decision {
    NoTransaction,
    Source(Direction),
    Target(Direction),
}

impl Decision {
    fn invert(self) -> Self {
        match self {
            Decision::NoTransaction => Decision::NoTransaction,
            Decision::Source(dir) => Decision::Target(-dir),
            Decision::Target(dir) => Decision::Source(-dir),
        }
    }
}

// Step 1: Each cell chooses a transaction to attempt
fn step1<Cell: Copy>(
    rule: &impl TransactionalCaRule<Cell = Cell>,
    nh: Neighbourhood<Cell>,
    rng: &mut SimRng,
) -> Decision {
    let center_as_source = nh.iter_dirs().filter_map(|(direction, neighbour)| {
        rule.transaction(nh.center, neighbour, direction)
            .map(|_result| Decision::Source(direction))
    });
    let center_as_target = nh.iter_dirs().filter_map(|(direction, neighbour)| {
        rule.transaction(neighbour, nh.center, -direction)
            .map(|_result| Decision::Target(direction))
    });
    let choices = center_as_source.chain(center_as_target);
    choices.choose(rng).unwrap_or(Decision::NoTransaction)
}

// Step 2: Execute transactions where both cells agree
fn step2<Cell: Copy>(
    rule: &impl TransactionalCaRule<Cell = Cell>,
    cells: Neighbourhood<Cell>,
    decisions: Neighbourhood<Decision>,
) -> Cell {
    match decisions.center {
        Decision::NoTransaction => cells.center,
        Decision::Source(direction) => {
            let idx = direction as usize;
            if decisions.center.invert() == decisions.neighbours[idx] {
                rule.transaction(cells.center, cells.neighbours[idx], direction)
                    .unwrap()
                    .source
            } else {
                cells.center
            }
        }
        Decision::Target(direction) => {
            let idx = direction as usize;
            if decisions.center.invert() == decisions.neighbours[idx] {
                rule.transaction(cells.neighbours[idx], cells.center, -direction)
                    .unwrap()
                    .target
            } else {
                cells.center
            }
        }
    }
}

pub fn step_torus<Rule: TransactionalCaRule>(
    tile: &TorusTile<Rule::Cell>,
    rule: &Rule,
    rng: &mut SimRng,
) -> TorusTile<Rule::Cell> {
    // optimize: depending on `f`, each step may depend on the previous step's
    // data through the RNG state. Independent RNGs may be better (or no RNGs)
    // (...and probably a lot more non-optimal stuff here)

    let step1: TorusTile<Decision> = tile
        .iter_radius_1()
        .map(|nh| step1(rule, nh, rng))
        .collect();

    let step2: TorusTile<Rule::Cell> = tile
        .iter_radius_1()
        .zip(step1.iter_radius_1())
        .map(|(cells, decisions)| step2(rule, cells, decisions))
        .collect();

    // Step 3: normal CA rules
    step2.iter_radius_1().map(|nh| rule.step(nh, rng)).collect()
}

pub fn step_axial<Rule: TransactionalCaRule>(
    tile: &AxialTile<Rule::Cell>,
    border: Rule::Cell,
    rule: &Rule,
    rng: &mut SimRng,
) -> AxialTile<Rule::Cell> {
    let step1: AxialTile<(Rule::Cell, Decision)> = tile
        .ca_step((border, Decision::NoTransaction), |nh| {
            (nh.center, step1(rule, nh, rng))
        });

    let step2: AxialTile<Rule::Cell> = step1.ca_step(border, |nh| {
        step2(
            rule,
            Neighbourhood {
                center: nh.center.0,
                neighbours: nh.neighbours.map(|(c, _)| c),
            },
            Neighbourhood {
                center: nh.center.1,
                neighbours: nh.neighbours.map(|(_, d)| d),
            },
        )
    });

    // Step 3: normal CA rules
    step2.ca_step(border, |nh| rule.step(nh, rng))
}
