#[derive(Default, Clone, Copy)]
pub struct FeatureAccumulator {
    acc: i64,
    n: u32,
}

impl FeatureAccumulator {
    pub fn push(&mut self, value: i32) {
        self.acc += value as i64;
        self.n += 1;
    }
    pub fn push_weighted(&mut self, value: i32, weight: u16) {
        self.acc += value as i64;
        self.n += weight as u32;
    }
    pub fn merge(fa1: Self, fa2: Self) -> Self {
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
