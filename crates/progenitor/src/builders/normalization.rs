use nalgebra::SVector;

use super::nn::N_INPUTS;

static INPUTS_MEAN: [f32; N_INPUTS] = [
    0.121289685,
    0.30893925,
    0.13220733,
    0.31559232,
    0.22244966,
    0.4579497,
    0.10832473,
    0.07778913,
    0.0069089155,
    0.015267838,
    0.009382486,
    0.0052882964,
    0.24600264,
    0.22745036,
    0.24872127,
    0.19521901,
    0.21195821,
    0.16440651,
    0.03352091,
    0.10285513,
    5.121973,
    -1.2788532,
    0.8057733,
    0.39936274,
    -1.8610246,
];

static INPUTS_STD_INV: [f32; N_INPUTS] = [
    1.0 / (0.1 + 0.3264645),
    1.0 / (0.1 + 0.4620567),
    1.0 / (0.1 + 0.33871612),
    1.0 / (0.1 + 0.46475098),
    1.0 / (0.1 + 0.41589156),
    1.0 / (0.1 + 0.49822888),
    1.0 / (0.1 + 0.31079045),
    1.0 / (0.1 + 0.26783958),
    1.0 / (0.1 + 0.08283215),
    1.0 / (0.1 + 0.12261547),
    1.0 / (0.1 + 0.09640818),
    1.0 / (0.1 + 0.072528206),
    1.0 / (0.1 + 0.15374064),
    1.0 / (0.1 + 0.1785075),
    1.0 / (0.1 + 0.1378129),
    1.0 / (0.1 + 0.1511757),
    1.0 / (0.1 + 0.16022283),
    1.0 / (0.1 + 0.14095944),
    1.0 / (0.1 + 0.17999248),
    1.0 / (0.1 + 0.15537864),
    1.0 / (0.1 + 4.998518),
    1.0 / (0.1 + 1.1572887),
    1.0 / (0.1 + 0.75167364),
    1.0 / (0.1 + 0.7827594),
    1.0 / (0.1 + 1.3937815),
];

pub fn normalize_inputs(inputs: &mut SVector<f32, N_INPUTS>) {
    let mean = SVector::<f32, N_INPUTS>::from_row_slice(&INPUTS_MEAN);
    let std_inv = SVector::<f32, N_INPUTS>::from_row_slice(&INPUTS_STD_INV);
    *inputs -= mean;
    inputs.component_mul_assign(&std_inv);

    // let std1: SVector<f32, N_INPUTS> = SVector::from_row_slice(&INPUTS_STD);
    // let std2 = std1.apply_into(|&mut v| v = 1. / (0.1 + v));
    // inputs.component_mul_assign(&std2);
}
