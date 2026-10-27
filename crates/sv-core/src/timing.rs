//! Frame-time predictor (milestone H2.1, initial version).
//!
//! Moving mean and variance with exponential decay. It is the baseline: if a
//! finer predictor does not beat it in measurements, it is not justified.

/// Exponential estimator of the mean and spread of the frame time.
#[derive(Debug, Clone)]
pub struct FrameTimePredictor {
    alpha: f64,
    mean: f64,
    var: f64,
    samples: u64,
}

impl FrameTimePredictor {
    /// `alpha` in (0, 1]: weight of the new sample. Lower values smooth more.
    ///
    /// # Panics
    /// If `alpha` is outside (0, 1].
    #[must_use]
    pub fn new(alpha: f64) -> Self {
        assert!(alpha > 0.0 && alpha <= 1.0, "alpha must be in (0, 1]");
        Self {
            alpha,
            mean: 0.0,
            var: 0.0,
            samples: 0,
        }
    }

    /// Adds a frame time, in ms. Ignores non-finite or negative values.
    pub fn observe(&mut self, frame_ms: f64) {
        if !frame_ms.is_finite() || frame_ms < 0.0 {
            return;
        }
        if self.samples == 0 {
            self.mean = frame_ms;
            self.var = 0.0;
        } else {
            let diff = frame_ms - self.mean;
            let incr = self.alpha * diff;
            self.mean += incr;
            self.var = (1.0 - self.alpha) * (self.var + diff * incr);
        }
        self.samples += 1;
    }

    /// Expected time of the next frame, in ms. `None` with no samples.
    #[must_use]
    pub fn predict_ms(&self) -> Option<f64> {
        (self.samples > 0).then_some(self.mean)
    }

    /// Estimated standard deviation, in ms.
    #[must_use]
    pub fn jitter_ms(&self) -> f64 {
        self.var.max(0.0).sqrt()
    }

    #[must_use]
    pub fn samples(&self) -> u64 {
        self.samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_predictor_has_no_prediction() {
        assert_eq!(FrameTimePredictor::new(0.2).predict_ms(), None);
    }

    #[test]
    fn constant_input_converges_with_no_jitter() {
        let mut p = FrameTimePredictor::new(0.2);
        for _ in 0..100 {
            p.observe(16.0);
        }
        assert!((p.predict_ms().unwrap() - 16.0).abs() < 1e-9);
        assert!(p.jitter_ms() < 1e-9);
    }

    #[test]
    fn follows_a_step_change() {
        let mut p = FrameTimePredictor::new(0.25);
        for _ in 0..50 {
            p.observe(10.0);
        }
        for _ in 0..50 {
            p.observe(20.0);
        }
        assert!((p.predict_ms().unwrap() - 20.0).abs() < 0.01);
    }

    #[test]
    fn alternating_input_reports_jitter() {
        let mut p = FrameTimePredictor::new(0.1);
        for i in 0..400 {
            p.observe(if i % 2 == 0 { 10.0 } else { 20.0 });
        }
        assert!(p.jitter_ms() > 3.0, "jitter = {}", p.jitter_ms());
    }

    #[test]
    fn ignores_invalid_samples() {
        let mut p = FrameTimePredictor::new(0.5);
        p.observe(f64::NAN);
        p.observe(-1.0);
        p.observe(f64::INFINITY);
        assert_eq!(p.samples(), 0);
    }

    #[test]
    #[should_panic(expected = "alpha")]
    fn rejects_bad_alpha() {
        let _ = FrameTimePredictor::new(0.0);
    }
}
