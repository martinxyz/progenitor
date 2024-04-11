#![allow(dead_code)]

// later, maybe: (and especially in case the NNs get larger: enable blas)
// use ndarray::prelude::*;
use nalgebra::{SMatrix, SVector};
use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::stats::RangeTracker;

pub const N_INPUTS: usize = 4 * 6 /* eye */ + 3 /* special */ + 8 /* memory */;
const N_HIDDEN: usize = 20;
const N_HIDDEN2: usize = 10;
pub const N_OUTPUTS: usize = 10 /* actions */ + 8 /* memory */;

pub struct Network {
    weights: Weights,
}

struct Weights {
    l1_w: SMatrix<f32, N_HIDDEN, N_INPUTS>,
    l1_b: SVector<f32, N_HIDDEN>,
    l2_w: SMatrix<f32, N_HIDDEN2, N_HIDDEN>,
    l2_b: SVector<f32, N_HIDDEN2>,
    o_w: SMatrix<f32, N_OUTPUTS, N_HIDDEN2>,
    o_b: SVector<f32, N_OUTPUTS>,
}

#[rustfmt::skip]
const PARAM_COUNT: usize =
    N_HIDDEN * N_INPUTS + N_HIDDEN +
    N_HIDDEN2 * N_HIDDEN + N_HIDDEN2 +
    N_OUTPUTS * N_HIDDEN2 + N_OUTPUTS;

fn relu(value: f32) -> f32 {
    f32::max(0., value)
}

// This becomes the bottleneck if the NN is large-ish (~30k weights), and the
// rest of the simulation doesn't do much. Could do fused multiply-accumulate,
// for starters. Could do integers instead of floats. Batching would be possible
// only when doing multiple parallel evals with the same weights.
// Note: nalgebra will use matrixmultiply() only if the matrix is "large", and
// defines "large" as "not compile-time sized".
fn forward(
    params: &Weights,
    inputs: [f32; N_INPUTS],
    update_statistics: impl FnOnce(ForwardTrace),
) -> SVector<f32, N_OUTPUTS> {
    let inputs: SVector<f32, N_INPUTS> = inputs.into();
    // normalize_inputs(&mut inputs); // FIXME: normalization helps. But besides being an ugly quick hack-implementation, it also doesn't need to be here where it costs ~5% of overall performance

    // // first layer
    let a1 = params.l1_w * inputs + params.l1_b;
    let a1 = a1.map(relu);
    let a2 = params.l2_w * a1 + params.l2_b;
    let a2 = a2.map(relu);
    // output layer
    let outputs = params.o_w * a2 + params.o_b;
    update_statistics(ForwardTrace {
        inputs,
        // a1,
        // outputs,
    });
    outputs
}

struct ForwardTrace {
    inputs: SVector<f32, N_INPUTS>,
    // a1: SVector<f32, N_HIDDEN>,
    // outputs: SVector<f32, N_OUTPUTS>,
}

#[derive(Default)]
struct Statistics {
    inputs: RangeTracker<N_INPUTS>,
    //     a1: RangeTracker<N_HIDDEN>,
    //     outputs: RangeTracker<N_OUTPUTS>,
}

/*
pub fn softmax_probs<const N: usize>(x: SVector<f32, N>) -> SVector<f32, N> {
    let mut x = x;
    let max = x.max();
    x.apply(|v| *v = (*v - max).exp());
    x /= x.sum();
    x
}
*/

#[derive(Clone, Serialize, Deserialize)]
pub struct Hyperparams {
    pub init_fac: f32,
    pub init_fac2: f32,
    pub bias_fac: f32,
    pub n_hidden: usize,
    pub n_hidden2: usize,
}

impl Hyperparams {
    pub fn count_params(&self) -> usize {
        self.validate().expect("hyperparams should be valid");
        self.n_hidden * N_INPUTS
            + self.n_hidden
            + self.n_hidden2 * self.n_hidden
            + self.n_hidden2
            + N_OUTPUTS * self.n_hidden2
            + N_OUTPUTS
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.n_hidden > N_HIDDEN {
            Err(format!(
                "n_hidden {} too large, nn was compiled for at most {}",
                self.n_hidden, N_HIDDEN
            ))
        } else if self.n_hidden2 > N_HIDDEN2 {
            Err(format!(
                "n_hidden2 {} too large, nn was compiled for at most {}",
                self.n_hidden2, N_HIDDEN2
            ))
        } else {
            Ok(())
        }
    }
}

impl Network {
    pub fn new(params: &[f32], hp: Hyperparams) -> Self {
        Network {
            weights: init_weights(params, hp),
            // stats: Default::default(),
        }
    }

    pub fn forward(&mut self, inputs: [f32; N_INPUTS]) -> SVector<f32, N_OUTPUTS> {
        forward(&self.weights, inputs, |_| {})
        // forward(&self.weights, inputs, |trace| {
        // self.stats.inputs.track(&trace.inputs);
        // self.stats.a1.track(&trace.a1);
        // self.stats.outputs.track(&trace.outputs);
        // })
    }

    pub fn print_stats(&self) {
        todo!()
        // println!(" inputs: {:?}", self.stats.inputs);
        // println!("     a1: {:?}", self.stats.a1);
        // println!("outputs: {:?}", self.stats.outputs);
    }
}

// Performance: this is sometimes a bottleneck. Time is spent in exp().
// Things tried:
// - The fastapprox crate's faster::exp() made it slower.
// - Much faster, but wrong: `max - *v + 1.0`
// - Faster by at least ~2x: `let x = *v - max; 1. / (1. + x*x)`
// - Faster, but not by much: `let x = *v - max; if x < -30. {0.} else {1. / (1. + x*x)}`
// - For large N: https://proceedings.neurips.cc/paper_files/paper/2017/file/4e2a6330465c8ffcaa696a5a16639176-Paper.pdf
// - Or maybe don't worry. It may become irrelevant as the simulation gets more fancy.
// - Or maybe don't optimize softmax, try something else and see how it trains.
#[allow(clippy::useless_conversion)]
pub fn softmax_choice<const N: usize>(outputs: SVector<f32, N>, rng: &mut impl Rng) -> usize {
    for v in outputs.iter() {
        assert!(v.is_finite(), "output[_] = {}", v);
    }
    let mut x = outputs;
    let max = x.max();
    x.apply(|v| *v = (*v - max).exp());
    WeightedIndex::new(x.into_iter()).unwrap().sample(rng)
}

fn init_weights(params: &[f32], hp: Hyperparams) -> Weights {
    hp.validate().expect("hyperparams should be valid");
    assert_eq!(
        params.len(),
        hp.count_params(),
        "param count does not match hyperparams"
    );
    let mut it = params.iter();
    let mut next_param = || it.next().unwrap();

    // For tuning the hidden layer size, we just set unused weights to zero.
    // (This leaves some performance on the table. However making the weights
    // dynamically sized did lead to a 3x slowdown due to missed vectorization.
    // So we'll just update the constants to match the plausible range found by
    // hyperparam search.)

    // something like Xavier and He initialization: https://stats.stackexchange.com/a/393012/52418
    #[rustfmt::skip]
    let weights = Weights {
        l1_w: SMatrix::from_fn(|i_hidden, _i_input| if i_hidden < hp.n_hidden {
            next_param() * (hp.init_fac * 2.0 / N_INPUTS as f32).sqrt()
        } else { 0.0 }),
        l1_b: SVector::from_fn(|i_hidden, _| if i_hidden < hp.n_hidden {
            next_param() * hp.init_fac * hp.bias_fac
        } else { 0.0 }),
        l2_w: SMatrix::from_fn(|i_hidden2, i_hidden| if i_hidden2 < hp.n_hidden2 && i_hidden < hp.n_hidden {
            next_param() * (hp.init_fac2 * hp.init_fac * 1.0 / (hp.n_hidden + hp.n_hidden2) as f32).sqrt()
        } else { 0.0 }),
        l2_b: SVector::from_fn(|i_hidden2, _| if i_hidden2 < hp.n_hidden2 {
            next_param() * hp.init_fac2 * hp.init_fac * hp.bias_fac
        } else { 0.0 }),
        o_w: SMatrix::from_fn(|_i_output, i_hidden2| if i_hidden2 < hp.n_hidden2 {
            next_param() * (hp.init_fac * 1.0 / (hp.n_hidden + N_OUTPUTS) as f32).sqrt()
        } else { 0.0 }),
        o_b: SVector::from_fn(|_i_output, _| {
            next_param() * hp.init_fac * hp.bias_fac
        }),
    };
    // (Bias: in contrast to what I'm doing above, pytorch defaults to
    // initialize biases the same as weights; maybe worth trying. Hyperparameter
    // search so far shows that bias is not too important, 0.1 is fine.)

    assert_eq!(it.count(), 0);
    weights
}
