use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::Rng;

pub struct Agent {
    action_distr: WeightedIndex<f32>,
}

pub fn dummy_agent() -> Agent {
    Agent {
        action_distr: WeightedIndex::new([
            // 4., // Forward
            // 1., // Left
            // 1., // Right
            // 2., // Pullback

            // optimized with CMA-ES:
            //
            // This is a super-silly strategy that seems to exploit the fact
            // that builders start on top of each other and face onyl six
            // distinct directions. Score: 5.83
            //
            2., // Forward
            0., // Left
            0., // Right
            1., // Pullback
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
