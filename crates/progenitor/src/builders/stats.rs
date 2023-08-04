use nalgebra::SVector;
use std::fmt::Debug;

pub struct RangeTracker<const N: usize> {
    count: usize,
    // https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford's_online_algorithm
    mean: SVector<f32, N>,
    m2: SVector<f32, N>,
}

impl<const N: usize> RangeTracker<N> {
    pub fn track(&mut self, v: &SVector<f32, N>) {
        self.count += 1;
        let delta = v - self.mean;
        self.mean += delta * (1. / self.count as f32);
        let delta2 = v - self.mean;
        self.m2 += delta.component_mul(&delta2);
    }

    fn means(&self) -> SVector<f32, N> {
        self.mean
    }

    fn stds(&self) -> SVector<f32, N> {
        let var = self.m2 / self.count as f32;
        var.map(f32::sqrt)
    }
}

impl<const N: usize> Default for RangeTracker<N> {
    fn default() -> Self {
        Self {
            count: 0,
            mean: SVector::zeros(),
            m2: SVector::zeros(),
        }
    }
}

impl<const N: usize> Debug for RangeTracker<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.count == 0 {
            write!(f, "[no samples collected; {} neurons]", N)
        } else {
            let means = self.means();
            let stds = self.stds();
            dbg!(means);
            dbg!(stds);
            write!(
                f,
                "[mean: {:.2}..{:.2}, std: {:.2}..{:.2}, N={}, {} neurons]",
                means.min(),
                means.max(),
                stds.min(),
                stds.max(),
                self.count,
                N
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_statistics() {
        let mut rt: RangeTracker<3> = Default::default();
        for i in 0..7 {
            rt.track(&Vector3::new(1., 2., i as f32))
        }
        assert!((rt.means()[0] - 1.).abs() < 0.0000001);
        assert!((rt.means()[1] - 2.).abs() < 0.0000001);
        assert_eq!(rt.stds()[0], 0.);
        assert_eq!(rt.stds()[1], 0.);
        assert!((rt.stds()[2] - 2.).abs() < 0.00000001);
    }

    #[test]
    fn test_nan_bug_reproducer() {
        let mut rt: RangeTracker<1> = Default::default();
        for _ in 0..30 {
            rt.track(&SVector::from_element(-500000.))
        }
        assert!(rt.stds()[0].is_finite());
        assert_eq!(rt.stds()[0], 0.);
    }

    #[test]
    fn test_numerical_stability() {
        let mut rt: RangeTracker<1> = Default::default();
        for _ in 0..30000 {
            rt.track(&SVector::from_element(-500000.))
        }
        assert!(rt.stds()[0].is_finite());
        assert_eq!(rt.stds()[0], 0.);
    }
}
