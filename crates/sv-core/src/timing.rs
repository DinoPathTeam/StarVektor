//! Predictor del tiempo de fotograma (hito H2.1, versión inicial).
//!
//! Media y varianza móviles con decaimiento exponencial. Es la línea base: si
//! un predictor más fino no la supera en las mediciones, no se justifica.

/// Estimador exponencial de la media y la dispersión del tiempo de fotograma.
#[derive(Debug, Clone)]
pub struct FrameTimePredictor {
    alpha: f64,
    mean: f64,
    var: f64,
    samples: u64,
}

impl FrameTimePredictor {
    /// `alpha` en (0, 1]: peso de la muestra nueva. Valores bajos suavizan más.
    ///
    /// # Panics
    /// Si `alpha` está fuera de (0, 1].
    #[must_use]
    pub fn new(alpha: f64) -> Self {
        assert!(alpha > 0.0 && alpha <= 1.0, "alpha debe estar en (0, 1]");
        Self {
            alpha,
            mean: 0.0,
            var: 0.0,
            samples: 0,
        }
    }

    /// Incorpora el tiempo de un fotograma, en ms. Ignora valores no finitos o negativos.
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

    /// Tiempo esperado del próximo fotograma, en ms. `None` sin muestras.
    #[must_use]
    pub fn predict_ms(&self) -> Option<f64> {
        (self.samples > 0).then_some(self.mean)
    }

    /// Desviación típica estimada, en ms.
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
