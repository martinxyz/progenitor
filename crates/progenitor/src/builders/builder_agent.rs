use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::Rng;

pub struct Agent {
    action_distr: WeightedIndex<f32>,
}

pub fn dummy_agent() -> Agent {
    Agent {
        action_distr: WeightedIndex::new([
            // hand-crafted
            // 4., // Forward
            // 1., // Left
            // 1., // Right
            // 2., // Pullback

            // optimized with CMA-ES for score after 50 steps: (score: 5.83)
            //
            // This is a super-silly strategy that seems to exploit the fact
            // that builders start on top of each other and face only six
            // distinct directions.
            //
            // 2., // Forward
            // 0., // Left
            // 0., // Right
            // 1., // Pullback

            // optimized for score after 1000 steps: (score: 6.65)
            0.237, // Forward
            0.299, // Left
            0.347, // Right
            0.115, // Pullback
        ])
        .unwrap(),
    }
}

pub fn categorical_agent(weights: &[f32]) -> Agent {
    assert_eq!(weights.len(), 4);
    Agent {
        action_distr: WeightedIndex::new(weights).unwrap(),
    }
}

impl Agent {
    pub fn act(&self, rng: &mut impl Rng) -> usize {
        self.action_distr.sample(rng)
    }
}
