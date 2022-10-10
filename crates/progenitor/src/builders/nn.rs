// later, maybe: (and especially in case the NNs get larger: enable blas)
// use ndarray::prelude::*;

use nalgebra::{SMatrix, SVector};
use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;

use super::stats::RangeTracker;

const N_INPUTS: usize = 6;
const N_HIDDEN: usize = 10;
const N_OUTPUTS: usize = 4;

pub struct Network {
    weights: Weights,
    stats: Statistics,
}

struct Weights {
    w0: SMatrix<f32, N_HIDDEN, N_INPUTS>,
    b0: SVector<f32, N_HIDDEN>,
    w1: SMatrix<f32, N_OUTPUTS, N_HIDDEN>,
    b1: SVector<f32, N_OUTPUTS>,
}

pub const PARAM_COUNT: usize = N_HIDDEN * N_INPUTS + N_HIDDEN + N_OUTPUTS * N_HIDDEN + N_OUTPUTS;

fn relu(value: f32) -> f32 {
    f32::max(0., value)
}

fn forward(
    params: &Weights,
    inputs: [f32; N_INPUTS],
    update_statistics: impl FnOnce(ForwardTrace),
) -> SVector<f32, N_OUTPUTS> {
    let inputs: SVector<f32, N_INPUTS> = inputs.into();
    // first layer
    let a1 = params.w0 * inputs + params.b0;
    let a2 = a1.map(relu);
    // output layer
    let outputs = params.w1 * a2 + params.b1;
    update_statistics(ForwardTrace {
        inputs,
        a1,
        outputs,
    });
    outputs
}

struct ForwardTrace {
    inputs: SVector<f32, N_INPUTS>,
    a1: SVector<f32, N_HIDDEN>,
    outputs: SVector<f32, N_OUTPUTS>,
}

#[derive(Default)]
struct Statistics {
    inputs: RangeTracker<N_INPUTS>,
    a1: RangeTracker<N_HIDDEN>,
    outputs: RangeTracker<N_OUTPUTS>,
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

impl Network {
    pub fn new(params: &[f32; PARAM_COUNT]) -> Self {
        Network {
            weights: init_weights(params),
            stats: Default::default(),
        }
    }

    pub fn forward(&mut self, inputs: [f32; N_INPUTS]) -> SVector<f32, N_OUTPUTS> {
        forward(&self.weights, inputs, |trace| {
            self.stats.inputs.track(&trace.inputs);
            self.stats.a1.track(&trace.a1);
            self.stats.outputs.track(&trace.outputs);
        })
    }

    pub fn print_stats(&self) {
        println!(" inputs: {:?}", self.stats.inputs);
        println!("     a1: {:?}", self.stats.a1);
        println!("outputs: {:?}", self.stats.outputs);
    }
}

// note: this seems to be the current performance bottleneck
pub fn softmax_choice<const N: usize>(outputs: SVector<f32, N>, rng: &mut impl Rng) -> usize {
    for v in outputs.iter() {
        assert!(v.is_finite(), "output[_] = {}", v);
    }
    let mut x = outputs;
    let max = x.max();
    x.apply(|v| *v = (*v - max).exp());
    WeightedIndex::new(x.into_iter()).unwrap().sample(rng)
}

fn init_weights(params: &[f32; PARAM_COUNT]) -> Weights {
    let mut it = params.iter();
    let mut next_param = || it.next().expect("PARAM_COUNT should match params length");

    // something like Xavier and He initialization: https://stats.stackexchange.com/a/393012/52418
    let weights = Weights {
        w0: SMatrix::from_fn(|_, _| next_param() * (2.0 / N_INPUTS as f32).sqrt()),
        b0: SVector::from_fn(|_, _| next_param() * 0.1),
        w1: SMatrix::from_fn(|_, _| next_param() * (1.0 / (N_HIDDEN + N_OUTPUTS) as f32).sqrt()),
        b1: SVector::from_fn(|_, _| next_param() * 0.1),
    };
    // (notes about biases: in contrast to what I'm doing above, pytorch
    // defaults to initialize biases the same as weights; maybe worth trying)

    assert_eq!(it.count(), 0, "PARAM_COUNT should match params length");
    weights
}
