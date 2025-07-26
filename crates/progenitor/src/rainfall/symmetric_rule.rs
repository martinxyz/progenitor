use hex2d::Angle;
use rand::Rng;
use rand_distr::{Beta, Distribution};
use serde::{Deserialize, Serialize};

use crate::{
    bit_particles::BitParticles,
    probabilities::{sigmoid, Prob15},
    Direction, DirectionSet, Neighbourhood,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct SymmetricRule {
    migration_requires_src_full: bool, // for blob-sticks-together behaviour
    migration_requires_dst_empty: bool,
    prob_shuffle: Prob15,
    prob_turn: Prob15,
    prob_reflect_in: Prob15,
    prob_reflect_out: Prob15,
}

impl SymmetricRule {
    pub fn sample(rng: &mut impl Rng) -> Self {
        Self::sample_from_params(rng, &[0.0; 10])
        // SymmetricRule {
        //     migration_requires_src_full: rng.random_bool(0.5),
        //     migration_requires_dst_empty: rng.random_bool(0.5),
        //     prob_shuffle: Prob15::HALF,
        //     prob_turn: Prob15::HALF,
        //     prob_reflect_in: Prob15::HALF,
        //     prob_reflect_out: Prob15::HALF,
        // }
    }

    pub fn sample_from_params(rng: &mut impl Rng, params: &[f32; 10]) -> Self {
        let mut params: Vec<_> = params.into();
        let mut param = || -> f32 {
            match params.pop() {
                Some(value) => value,
                None => panic!("too few distribution params"),
            }
        };

        let result = SymmetricRule {
            migration_requires_src_full: param_bool(rng, param()),
            migration_requires_dst_empty: param_bool(rng, param()),
            prob_shuffle: Prob15::from_p(param_prob(rng, param(), param())),
            prob_turn: Prob15::from_p(param_prob(rng, param(), param())),
            prob_reflect_in: Prob15::from_p(param_prob(rng, param(), param())),
            prob_reflect_out: Prob15::from_p(param_prob(rng, param(), param())),
        };
        assert_eq!(params.len(), 0, "too many distribution params");
        return result;

        fn param_bool(rng: &mut impl Rng, param: f32) -> bool {
            rng.random_bool(sigmoid(param).into())
        }

        fn param_prob(rng: &mut impl Rng, param1: f32, param2: f32) -> f32 {
            // alpha = 2*np.log(1+np.exp(param1))
            let alpha = (param1.exp() + 1.0).ln();
            let beta = (param2.exp() + 1.0).ln();
            let dist = Beta::new(alpha, beta).unwrap();
            dist.sample(rng)
        }
    }
}

impl SymmetricRule {
    pub fn step1_transfer(&self, nh: Neighbourhood<Option<BitParticles>>) -> BitParticles {
        let center = nh.center.unwrap();
        let outgoing = center.outgoing();

        let outgoing_next = DirectionSet::matching(|dir| {
            let has_outgoing = outgoing.has(dir);
            if let Some(neigh) = nh[dir] {
                let has_incoming = neigh.outgoing().has(-dir);
                if has_outgoing && !has_incoming {
                    // we keep the mass-particle if outgoing is not allowed
                    !self.outgoing_allowed(center.resting(), neigh.resting())
                } else if !has_outgoing && has_incoming {
                    // we gain the mass-particle if incoming is allowed
                    self.outgoing_allowed(neigh.resting(), center.resting())
                } else {
                    // no transfer (copy old state)
                    has_outgoing
                }
            } else {
                has_outgoing
            }
        });
        BitParticles::new(outgoing_next, center.resting())
    }

    fn outgoing_allowed(&self, src_resting: u8, dst_resting: u8) -> bool {
        if self.migration_requires_src_full && src_resting == 0 {
            false
        } else if self.migration_requires_dst_empty && dst_resting != 0 {
            false
        } else {
            true
        }
    }

    pub fn step2_shuffle(&self, particles: &mut BitParticles, rng: &mut impl Rng) {
        if self.prob_reflect_in.sample(rng) {
            particles.reflect_all();
        }

        if self.prob_shuffle.sample(rng) {
            // xxx sequential dependency (should use a forked rng stream)
            particles.shuffle8_cheap(rng);
        }

        if self.prob_turn.sample(rng) {
            // xxx sequential dependency (should use a forked rng stream)
            if let Ok(dir) = Direction::try_from(rng.random::<u8>() as i32) {
                particles.set_outgoing(particles.outgoing().swapped(dir, dir + Angle::Right));
            }
        }
        if self.prob_reflect_out.sample(rng) {
            particles.reflect_all();
        }
    }
}
