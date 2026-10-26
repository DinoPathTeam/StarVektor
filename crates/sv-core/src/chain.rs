//! Detección de la cadena: ¿hay upscaling, generación de fotogramas o ambos? (D7)
//!
//! Se detecta en vez de asumirse. La señal de generación de fotogramas es la
//! relación entre los fotogramas presentados y los que traen datos de
//! movimiento nuevos: si llegan datos nuevos cada dos fotogramas, hay FG.
//! Es una heurística: se valida en la fase 1 (H1.2) con OptiScaler real.

use sv_types::OptiScalerMode;

/// Medidas de la última ventana de observación.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChainSignals {
    /// Fotogramas presentados por segundo.
    pub presented_fps: f64,
    /// Fotogramas por segundo con vectores de movimiento nuevos.
    pub fresh_motion_fps: f64,
    /// Si se detectó un upscaler temporal en la cadena del juego.
    pub upscaler_detected: bool,
}

/// Relación presentados/nuevos a partir de la cual se considera FG activo.
const FG_RATIO_THRESHOLD: f64 = 1.7;

/// Clasificación instantánea, sin memoria.
#[must_use]
pub fn classify(signals: &ChainSignals) -> OptiScalerMode {
    let fg = signals.fresh_motion_fps > 0.0
        && signals.presented_fps / signals.fresh_motion_fps >= FG_RATIO_THRESHOLD;
    match (signals.upscaler_detected, fg) {
        (false, false) => OptiScalerMode::None,
        (true, false) => OptiScalerMode::Upscale,
        (false, true) => OptiScalerMode::Fg,
        (true, true) => OptiScalerMode::Both,
    }
}

/// Detector con confirmación: solo cambia de modo tras varias ventanas iguales,
/// para que una escena de carga o un corte no lo haga oscilar.
#[derive(Debug, Clone)]
pub struct ChainDetector {
    confirm_after: u32,
    current: OptiScalerMode,
    candidate: OptiScalerMode,
    streak: u32,
}

impl ChainDetector {
    /// `confirm_after`: ventanas consecutivas necesarias para aceptar un cambio.
    #[must_use]
    pub fn new(confirm_after: u32) -> Self {
        Self {
            confirm_after: confirm_after.max(1),
            current: OptiScalerMode::None,
            candidate: OptiScalerMode::None,
            streak: 0,
        }
    }

    /// Procesa una ventana y devuelve el modo vigente.
    pub fn observe(&mut self, signals: &ChainSignals) -> OptiScalerMode {
        let seen = classify(signals);
        if seen == self.current {
            self.streak = 0;
            self.candidate = self.current;
        } else if seen == self.candidate {
            self.streak += 1;
            if self.streak >= self.confirm_after {
                self.current = seen;
                self.streak = 0;
            }
        } else {
            self.candidate = seen;
            self.streak = 1;
            if self.streak >= self.confirm_after {
                self.current = seen;
                self.streak = 0;
            }
        }
        self.current
    }

    #[must_use]
    pub fn current(&self) -> OptiScalerMode {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(presented: f64, fresh: f64, upscaler: bool) -> ChainSignals {
        ChainSignals {
            presented_fps: presented,
            fresh_motion_fps: fresh,
            upscaler_detected: upscaler,
        }
    }

    #[test]
    fn classifies_the_four_modes() {
        assert_eq!(classify(&sig(60.0, 60.0, false)), OptiScalerMode::None);
        assert_eq!(classify(&sig(60.0, 60.0, true)), OptiScalerMode::Upscale);
        assert_eq!(classify(&sig(120.0, 60.0, false)), OptiScalerMode::Fg);
        assert_eq!(classify(&sig(120.0, 60.0, true)), OptiScalerMode::Both);
    }

    #[test]
    fn no_motion_data_is_not_fg() {
        assert_eq!(classify(&sig(60.0, 0.0, false)), OptiScalerMode::None);
    }

    #[test]
    fn detector_ignores_a_single_odd_window() {
        let mut d = ChainDetector::new(3);
        for _ in 0..5 {
            d.observe(&sig(60.0, 60.0, true));
        }
        assert_eq!(d.current(), OptiScalerMode::Upscale);
        // Una ventana rara (carga de escena) no cambia el modo.
        d.observe(&sig(120.0, 60.0, true));
        assert_eq!(d.current(), OptiScalerMode::Upscale);
        d.observe(&sig(60.0, 60.0, true));
        assert_eq!(d.current(), OptiScalerMode::Upscale);
    }

    #[test]
    fn detector_switches_after_confirmation() {
        let mut d = ChainDetector::new(3);
        for _ in 0..3 {
            d.observe(&sig(60.0, 60.0, true));
        }
        assert_eq!(d.current(), OptiScalerMode::Upscale);
        d.observe(&sig(120.0, 60.0, true));
        d.observe(&sig(120.0, 60.0, true));
        assert_eq!(d.current(), OptiScalerMode::Upscale);
        d.observe(&sig(120.0, 60.0, true));
        assert_eq!(d.current(), OptiScalerMode::Both);
    }

    #[test]
    fn confirm_after_one_switches_immediately() {
        let mut d = ChainDetector::new(1);
        assert_eq!(d.observe(&sig(120.0, 60.0, false)), OptiScalerMode::Fg);
    }
}
