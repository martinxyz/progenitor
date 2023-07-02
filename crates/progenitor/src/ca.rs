//! Cellular Automata Helpers

use rand::seq::IteratorRandom;

use crate::coords::Direction;
use crate::{AxialTile, SimRng, TorusTile};

// XXX should just use 'Neighbourhood' struct everywhere instead.
pub struct Neighbours<Cell: Copy>([(Direction, Cell); 6]);

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
    fn step(
        &self,
        center: Self::Cell,
        neighbours: Neighbours<Self::Cell>,
        rng: &mut SimRng,
    ) -> Self::Cell;
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

fn iter_dirs<Cell: Copy>(neighbours: [Cell; 6]) -> impl Iterator<Item = (Direction, Cell)> {
    Direction::all().into_iter().zip(neighbours)
}

// Step 1: Each cell chooses a transaction to attempt
fn step1<Cell: Copy>(
    rule: &impl TransactionalCaRule<Cell = Cell>,
    center: Cell,
    neighbours: [Cell; 6],
    rng: &mut SimRng,
) -> Decision {
    let center_as_source = iter_dirs(neighbours).filter_map(|(direction, neighbour)| {
        rule.transaction(center, neighbour, direction)
            .map(|_result| Decision::Source(direction))
    });
    let center_as_target = iter_dirs(neighbours).filter_map(|(direction, neighbour)| {
        rule.transaction(neighbour, center, -direction)
            .map(|_result| Decision::Target(direction))
    });
    let choices = center_as_source.chain(center_as_target);
    choices.choose(rng).unwrap_or(Decision::NoTransaction)
}

// Step 2: Execute transactions where both cells agree
fn step2<Cell: Copy>(
    rule: &impl TransactionalCaRule<Cell = Cell>,
    cell: Cell,
    cell_neighbours: [Cell; 6],
    decision: Decision,
    decision_neighbours: [Decision; 6],
) -> Cell {
    match decision {
        Decision::NoTransaction => cell,
        Decision::Source(direction) => {
            let idx = direction as usize;
            if decision.invert() == decision_neighbours[idx] {
                rule.transaction(cell, cell_neighbours[idx], direction)
                    .unwrap()
                    .source
            } else {
                cell
            }
        }
        Decision::Target(direction) => {
            let idx = direction as usize;
            if decision.invert() == decision_neighbours[idx] {
                rule.transaction(cell_neighbours[idx], cell, -direction)
                    .unwrap()
                    .target
            } else {
                cell
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
        .map(|(center, neighbours)| step1(rule, center, neighbours.map(|(_d, n)| n), rng))
        .collect();

    let step2: TorusTile<Rule::Cell> = tile
        .iter_radius_1()
        .zip(step1.iter_radius_1())
        .map(
            |((cell, cell_neighbours), (decision, decision_neighbours))| {
                step2(
                    rule,
                    cell,
                    cell_neighbours.map(|(_, n)| n),
                    decision,
                    decision_neighbours.map(|(_, n)| n),
                )
            },
        )
        .collect();

    // Step 3: normal CA rules
    step2
        .iter_radius_1()
        .map(|(center, neighbours)| rule.step(center, Neighbours(neighbours), rng))
        .collect()
}

pub fn step_axial<Rule: TransactionalCaRule>(
    tile: &AxialTile<Rule::Cell>,
    fill: Rule::Cell,
    rule: &Rule,
    rng: &mut SimRng,
) -> AxialTile<Rule::Cell> {
    let step1: AxialTile<(Rule::Cell, Decision)> = tile
        .ca_step((fill, Decision::NoTransaction), |n| {
            (n.center, step1(rule, n.center, n.neighbours, rng))
        });

    let step2: AxialTile<Rule::Cell> = step1.ca_step(fill, |n| {
        step2(
            rule,
            n.center.0,
            n.neighbours.map(|(c, _)| c),
            n.center.1,
            n.neighbours.map(|(_, d)| d),
        )
    });

    // Step 3: normal CA rules
    step2.ca_step(fill, |n| {
        rule.step(
            n.center,
            Neighbours(
                // Ugh. Just use Neighbourhood everywhere, really.
                iter_dirs(n.neighbours)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap_or_else(|_| panic!()),
            ),
            rng,
        )
    })
}
