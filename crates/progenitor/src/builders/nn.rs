#![allow(dead_code)]

// later, maybe: (and especially in case the NNs get larger: enable blas)
// use ndarray::prelude::*;
use nalgebra::{DMatrix, DVector};
use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::stats::RangeTracker;

pub const N_INPUTS: usize = 3 * 6 /* eye */ + 2 /* special */ + 10 /* last action */ + 4 /* memory */;
const N_HIDDEN: usize = 20;
const N_HIDDEN2: usize = 20;
pub const N_OUTPUTS: usize = 10 /* actions */ + 4 /* memory */;

pub struct Network {
    weights: Weights,
    allocations: Allocations,
}

struct Weights {
    l1_w: DMatrix<f32>,
    l1_b: DVector<f32>,
    l2_w: DMatrix<f32>,
    l2_b: DVector<f32>,
    o_w: DMatrix<f32>,
    o_b: DVector<f32>,
}

struct Allocations {
    l1: DVector<f32>,
    l2: DVector<f32>,
    outputs: DVector<f32>,
}

#[rustfmt::skip]
pub const PARAM_COUNT: usize =
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
fn forward<'t>(
    params: &Weights,
    inputs: DVector<f32>,
    allocations: &'t mut Allocations,
    update_statistics: impl FnOnce(ForwardTrace),
) -> &'t mut DVector<f32> {
    // normalize_inputs(&mut inputs); // FIXME: normalization helps. But besides being an ugly quick hack-implementation, it also doesn't need to be here where it costs ~5% of overall performance

    // FIXME: this does not run any faster than the simpler (no-gemv) version I
    // had before. (Still 3x slower than the compile-time sized version.) On the
    // plus side, it now spends all its time inside gemv, so maybe we can use a
    // properly optimized blas library? (Apparently not?)

    let a1 = &mut allocations.l1;
    let a2 = &mut allocations.l2;
    let outputs = &mut allocations.outputs;

    // first layer
    a1.copy_from(&params.l1_b);
    a1.gemv(1.0, &params.l1_w, &inputs, 1.0);
    a1.apply(|v| *v = relu(*v));

    // hidden layer
    a2.copy_from(&params.l2_b);
    a2.gemv(1.0, &params.l2_w, &a1, 1.0);

    // output layer
    outputs.copy_from(&params.o_b);
    outputs.gemv(1.0, &params.o_w, &a2, 1.0);

    update_statistics(ForwardTrace {
        inputs,
        // a1,
        // outputs,
    });
    outputs
}

struct ForwardTrace {
    inputs: DVector<f32>,
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
    pub bias_fac: f32,
}

impl Network {
    pub fn new(params: &DVector<f32>, hp: Hyperparams) -> Self {
        Network {
            weights: init_weights(params, hp),
            allocations: Allocations {
                l1: DVector::zeros(N_HIDDEN),
                l2: DVector::zeros(N_HIDDEN2),
                outputs: DVector::zeros(N_OUTPUTS),
            }, // stats: Default::default(),
        }
    }

    pub fn forward(&mut self, inputs: DVector<f32>) -> &mut DVector<f32> {
        forward(&self.weights, inputs, &mut self.allocations, |_| {})
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
// (need to re-eval after converting go DVector)
pub fn softmax_choice(outputs: DVector<f32>, rng: &mut impl Rng) -> usize {
    for v in outputs.iter() {
        assert!(v.is_finite(), "output[_] = {}", v);
    }
    let mut x = outputs;
    let max = x.max();
    x.apply(|v| *v = (*v - max).exp());
    WeightedIndex::new(&x).unwrap().sample(rng)
}

fn init_weights(params: &DVector<f32>, hp: Hyperparams) -> Weights {
    assert_eq!(
        PARAM_COUNT,
        params.len(),
        "PARAM_COUNT should match params length"
    );
    let mut it = params.iter();
    let mut next_param = || it.next().expect("PARAM_COUNT should match params length");

    // something like Xavier and He initialization: https://stats.stackexchange.com/a/393012/52418
    #[rustfmt::skip]
    let weights = Weights {
        l1_w: DMatrix::from_fn(N_HIDDEN, N_INPUTS, |_, _| next_param() * (hp.init_fac * 2.0 / N_INPUTS as f32).sqrt()),
        l1_b: DVector::from_fn(N_HIDDEN, |_, _| next_param() * hp.init_fac * hp.bias_fac),
        l2_w: DMatrix::from_fn(N_HIDDEN2, N_HIDDEN, |_, _| next_param() * (hp.init_fac * 1.0 / (N_HIDDEN + N_HIDDEN2) as f32).sqrt()),
        l2_b: DVector::from_fn(N_HIDDEN2, |_, _| next_param() * hp.init_fac * hp.bias_fac),
        o_w: DMatrix::from_fn(N_OUTPUTS, N_HIDDEN2, |_, _| next_param() * (hp.init_fac * 1.0 / (N_HIDDEN2 + N_OUTPUTS) as f32).sqrt()),
        o_b: DVector::from_fn(N_OUTPUTS, |_, _| next_param() * hp.init_fac * hp.bias_fac),
    };
    // (Bias: in contrast to what I'm doing above, pytorch defaults to
    // initialize biases the same as weights; maybe worth trying. Hyperparameter
    // search so far shows that bias is not too important, 0.1 is fine.)

    assert_eq!(it.count(), 0, "PARAM_COUNT should match params length");
    weights
}
