// later, maybe: (and especially in case the NNs get larger: enable blas)
// use ndarray::prelude::*;
use nalgebra::{SMatrix, SVector};
use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;
use serde::{Deserialize, Serialize};

// use super::stats::RangeTracker;

const N_INPUTS: usize = 2 * 6 + 2 /* eye */ + 4 /* special */ + 4 /* memory */;
const N_HIDDEN: usize = 15;
const N_HIDDEN2: usize = 15;
const N_OUTPUTS: usize = 4 /* actions */ + 4 /* memory */;

pub struct Network {
    weights: Weights,
    // stats: Statistics,
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
fn forward(
    params: &Weights,
    inputs: [f32; N_INPUTS],
    // update_statistics: impl FnOnce(ForwardTrace),
) -> SVector<f32, N_OUTPUTS> {
    let inputs: SVector<f32, N_INPUTS> = inputs.into();
    // // first layer
    let a1 = params.l1_w * inputs + params.l1_b;
    let a1 = a1.map(relu);
    let a2 = params.l2_w * a1 + params.l2_b;
    let a2 = a2.map(relu);
    // output layer
    let outputs = params.o_w * a2 + params.o_b;
    // update_statistics(ForwardTrace {
    //     inputs,
    //     a1,
    //     outputs,
    // });
    outputs
}

// struct ForwardTrace {
//     inputs: SVector<f32, N_INPUTS>,
//     a1: SVector<f32, N_HIDDEN>,
//     outputs: SVector<f32, N_OUTPUTS>,
// }

// #[derive(Default)]
// struct Statistics {
//     inputs: RangeTracker<N_INPUTS>,
//     a1: RangeTracker<N_HIDDEN>,
//     outputs: RangeTracker<N_OUTPUTS>,
// }

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
    pub fn new(params: &[f32; PARAM_COUNT], hp: Hyperparams) -> Self {
        Network {
            weights: init_weights(params, hp),
            // stats: Default::default(),
        }
    }

    pub fn forward(&mut self, inputs: [f32; N_INPUTS]) -> SVector<f32, N_OUTPUTS> {
        forward(&self.weights, inputs)
        // forward(&self.weights, inputs, |trace| {
        //     self.stats.inputs.track(&trace.inputs);
        //     self.stats.a1.track(&trace.a1);
        //     self.stats.outputs.track(&trace.outputs);
        // })
    }

    pub fn print_stats(&self) {
        todo!()
        // println!(" inputs: {:?}", self.stats.inputs);
        // println!("     a1: {:?}", self.stats.a1);
        // println!("outputs: {:?}", self.stats.outputs);
    }
}

// note: this seems to be the current performance bottleneck
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

fn init_weights(params: &[f32; PARAM_COUNT], hp: Hyperparams) -> Weights {
    let mut it = params.iter();
    let mut next_param = || it.next().expect("PARAM_COUNT should match params length");

    // something like Xavier and He initialization: https://stats.stackexchange.com/a/393012/52418
    #[rustfmt::skip]
    let weights = Weights {
        l1_w: SMatrix::from_fn(|_, _| next_param() * (hp.init_fac * 2.0 / N_INPUTS as f32).sqrt()),
        l1_b: SVector::from_fn(|_, _| next_param() * hp.init_fac * hp.bias_fac),
        l2_w: SMatrix::from_fn(|_, _| next_param() * (hp.init_fac * 1.0 / (N_HIDDEN + N_HIDDEN2) as f32).sqrt()),
        l2_b: SVector::from_fn(|_, _| next_param() * hp.init_fac * hp.bias_fac),
        o_w: SMatrix::from_fn(|_, _| next_param() * (hp.init_fac * 1.0 / (N_HIDDEN2 + N_OUTPUTS) as f32).sqrt()),
        o_b: SVector::from_fn(|_, _| next_param() * hp.init_fac * hp.bias_fac),
    };
    // (Bias: in contrast to what I'm doing above, pytorch defaults to
    // initialize biases the same as weights; maybe worth trying. Hyperparameter
    // search so far shows that bias is not too important, 0.1 is fine.)

    assert_eq!(it.count(), 0, "PARAM_COUNT should match params length");
    weights
}
