//! Chain detection: is there upscaling, frame generation, or both? (D7)
//!
//! It is detected instead of assumed. The frame generation signal is the ratio
//! between presented frames and frames that carry new motion data: if new data
//! arrives every second frame, FG is active. This is a heuristic: it is
//! validated in Phase 1 (H1.2) with a real OptiScaler.

use sv_types::OptiScalerMode;

/// Measurements from the latest observation window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChainSignals {
    /// Frames presented per second.
    pub presented_fps: f64,
    /// Frames per second with fresh motion vectors.
    pub fresh_motion_fps: f64,
    /// Whether a temporal upscaler was detected in the game's chain.
    pub upscaler_detected: bool,
}

/// Presented/fresh ratio from which FG is considered active.
const FG_RATIO_THRESHOLD: f64 = 1.7;

/// Instantaneous classification, with no memory.
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

/// Detector with confirmation: it only changes mode after several identical
/// windows, so a loading screen or a cut does not make it oscillate.
#[derive(Debug, Clone)]
pub struct ChainDetector {
    confirm_after: u32,
    current: OptiScalerMode,
    candidate: OptiScalerMode,
    streak: u32,
}

impl ChainDetector {
    /// `confirm_after`: consecutive windows needed to accept a change.
    #[must_use]
    pub fn new(confirm_after: u32) -> Self {
        Self {
            confirm_after: confirm_after.max(1),
            current: OptiScalerMode::None,
            candidate: OptiScalerMode::None,
            streak: 0,
        }
    }

    /// Processes a window and returns the current mode.
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
        // A single odd window (scene load) does not change the mode.
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
