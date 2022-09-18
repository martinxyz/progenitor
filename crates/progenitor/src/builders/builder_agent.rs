use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::Rng;

pub struct Agent {
    action_distr: WeightedIndex<i32>,
}

pub fn dummy_agent() -> Agent {
    Agent {
        action_distr: WeightedIndex::new([
            4, // Forward
            1, // Left
            1, // Right
            2, // Pullback
        ])
        .unwrap(),
    }
}

impl Agent {
    pub fn act(&self, rng: &mut impl Rng) -> usize {
        self.action_distr.sample(rng)
    }
}
