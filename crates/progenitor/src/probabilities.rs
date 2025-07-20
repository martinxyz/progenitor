use rand::Rng;
use serde::{Deserialize, Serialize};

pub fn logit(p: f32) -> f32 {
    let p = p.clamp(0.00001, 0.99999);
    (p / (1.0 - p)).ln()
}

pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Prob15(u16);

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Prob7(u8);

impl Prob7 {
    pub const ZERO: Self = Self(0);
    pub const HALF: Self = Self(64);
    pub const ONE: Self = Self(128);
    pub fn from_logit(logit: f32) -> Self {
        assert!(!logit.is_nan());
        Self::from_p(sigmoid(logit))
    }
    pub fn from_p(p: f32) -> Self {
        assert!(p.is_finite());
        assert!(p > -0.001);
        assert!(p < 1.001);
        Self((p * 128.0).round().clamp(0.0, 128.0) as u8)
    }
    pub fn sample(self, rng: &mut impl Rng) -> bool {
        rng.random::<u8>() & 0b01111111 < self.0
    }
}

impl Prob15 {
    pub const ZERO: Self = Self(0);
    pub const HALF: Self = Self(1 << 14);
    pub const ONE: Self = Self(1 << 15);
    pub fn from_logit(logit: f32) -> Self {
        assert!(!logit.is_nan());
        Self::from_p(sigmoid(logit))
    }
    pub fn from_p(p: f32) -> Self {
        assert!(p.is_finite());
        assert!(p > -0.001);
        assert!(p < 1.001);
        Self(
            (p * ((1 << 15) as f32))
                .round()
                .clamp(0.0, (1 << 15) as f32) as u16,
        )
    }
    pub fn sample(self, rng: &mut impl Rng) -> bool {
        rng.random::<u16>() & 0x7FFF < self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimRng;
    use rand::SeedableRng;

    #[test]
    fn test_prob_7() {
        let mut rng = SimRng::seed_from_u64(42);
        const N: u64 = 1000;
        let p_zero = Prob7::from_p(0.0);
        let p_one = Prob7::from_p(1.0);
        let none = (0..N).map(|_| p_zero.sample(&mut rng) as u64).sum::<u64>();
        let all = (0..N).map(|_| p_one.sample(&mut rng) as u64).sum::<u64>();
        assert_eq!(none, 0);
        assert_eq!(all, N);

        let p_half = Prob7::from_p(0.5);
        let about_half = (0..N).map(|_| p_half.sample(&mut rng) as u64).sum::<u64>();
        assert!(about_half > N * 3 / 8);
        assert!(about_half < N * 5 / 8);
    }

    #[test]
    fn test_prob_15() {
        let mut rng = SimRng::seed_from_u64(42);
        const N: u64 = 1000;
        let p_zero = Prob15::from_p(0.0);
        let p_one = Prob15::from_p(1.0);
        let none = (0..N).map(|_| p_zero.sample(&mut rng) as u64).sum::<u64>();
        let all = (0..N).map(|_| p_one.sample(&mut rng) as u64).sum::<u64>();
        assert_eq!(none, 0);
        assert_eq!(all, N);

        let p_half = Prob15::from_p(0.5);
        let about_half = (0..N).map(|_| p_half.sample(&mut rng) as u64).sum::<u64>();
        assert!(about_half > N * 3 / 8);
        assert!(about_half < N * 5 / 8);
    }
}
