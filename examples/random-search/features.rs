use progenitor::{Cell, CellTypeRef, World};

#[derive(Default, Clone, Copy)]
struct FeatureAccumulator {
    acc: i64,
    n: u32,
}

impl FeatureAccumulator {
    fn push(&mut self, value: i32) {
        self.acc += value as i64;
        self.n += 1;
    }
    fn push_weighted(&mut self, value: i32, weight: u16) {
        self.acc += value as i64;
        self.n += weight as u32;
    }
    fn merge((fa1, fa2): (Self, Self)) -> Self {
        Self {
            acc: fa1.acc + fa2.acc,
            n: fa1.n + fa2.n,
        }
    }
}

impl From<FeatureAccumulator> for f64 {
    fn from(fa: FeatureAccumulator) -> Self {
        fa.acc as f64 / fa.n as f64
    }
}

pub const FEATURE_COUNT: usize = 2;

fn calculate_features(world: World) -> [FeatureAccumulator; FEATURE_COUNT] {
    const EMPTY: CellTypeRef = CellTypeRef(1);
    fn cell2int(c: Cell) -> i32 {
        if c.cell_type == EMPTY { 0 } else { 1 }
    }
    let mut features = [FeatureAccumulator::default(); FEATURE_COUNT];
    for (cell, neighbours) in world.iter_cells_with_neighbours() {
        let center = cell2int(cell);
        let neighbours: i32 = neighbours.iter().map(|(_, c)| cell2int(*c)).sum();
        features[0].push(center);
        features[1].push_weighted((neighbours - 6 * center).abs(), 6);
    }
    features
}

pub fn evaluate<F>(run: F, repetitions: i32) -> [f64; FEATURE_COUNT]
where F: Fn() -> World
{
    (0..repetitions)
        .map(|_| run())
        .map(calculate_features)
        .reduce(|a, b| a.zip(b).map(FeatureAccumulator::merge))
        .unwrap()
        .map(|fa| fa.into())
}
