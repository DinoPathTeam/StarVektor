//! Synthetic platform: generates a fictitious game with controlled frame
//! times. It is deterministic (same seed, same sequence).

use sv_types::{FrameHandle, Resolution};

use crate::{CapturedFrame, Platform, PlatformError, PresentRequest};

/// Parameters of the fictitious game.
#[derive(Debug, Clone)]
pub struct SyntheticConfig {
    /// Mean frame time of the "game", in ms.
    pub base_frame_ms: f64,
    /// Maximum variation around the mean (±), in ms.
    pub jitter_ms: f64,
    pub refresh_hz: f64,
    /// How many frames it delivers before finishing.
    pub frames: u64,
    pub resolution: Resolution,
    pub seed: u64,
    /// Frame indices at which "the user presses the mark key".
    pub marks_at: Vec<u64>,
}

impl Default for SyntheticConfig {
    fn default() -> Self {
        Self {
            base_frame_ms: 22.0,
            jitter_ms: 2.0,
            refresh_hz: 60.0,
            frames: 600,
            resolution: Resolution(2560, 1440),
            seed: 1,
            marks_at: Vec::new(),
        }
    }
}

pub struct SyntheticPlatform {
    cfg: SyntheticConfig,
    rng: XorShift64,
    produced: u64,
    clock_ms: f64,
    pending_mark: bool,
}

impl SyntheticPlatform {
    #[must_use]
    pub fn new(cfg: SyntheticConfig) -> Self {
        let rng = XorShift64::new(cfg.seed);
        Self {
            cfg,
            rng,
            produced: 0,
            clock_ms: 0.0,
            pending_mark: false,
        }
    }
}

impl Platform for SyntheticPlatform {
    fn name(&self) -> &'static str {
        "synthetic"
    }

    fn refresh_period_ms(&self) -> f64 {
        1000.0 / self.cfg.refresh_hz
    }

    fn next_frame(&mut self) -> Result<Option<CapturedFrame>, PlatformError> {
        if self.produced >= self.cfg.frames {
            return Ok(None);
        }
        let jitter = (self.rng.next_unit() * 2.0 - 1.0) * self.cfg.jitter_ms;
        self.clock_ms += (self.cfg.base_frame_ms + jitter).max(0.1);
        let index = self.produced;
        self.produced += 1;
        if self.cfg.marks_at.contains(&index) {
            self.pending_mark = true;
        }
        Ok(Some(CapturedFrame {
            handle: FrameHandle(index),
            index,
            render_ts_ms: self.clock_ms,
            resolution: self.cfg.resolution,
            fresh_motion: true,
        }))
    }

    fn present(&mut self, request: &PresentRequest) -> Result<f64, PlatformError> {
        Ok(request.target_ts_ms)
    }

    fn poll_user_mark(&mut self) -> bool {
        std::mem::take(&mut self.pending_mark)
    }
}

/// Minimal pseudo-random generator (xorshift64*), enough for tests.
struct XorShift64(u64);

impl XorShift64 {
    fn new(seed: u64) -> Self {
        // The state cannot be zero.
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    /// Number in [0, 1).
    fn next_unit(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let value = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (value >> 11) as f64 / (1_u64 << 53) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain(cfg: SyntheticConfig) -> Vec<CapturedFrame> {
        let mut p = SyntheticPlatform::new(cfg);
        std::iter::from_fn(|| p.next_frame().unwrap()).collect()
    }

    #[test]
    fn delivers_exactly_the_requested_frames_then_stops() {
        let frames = drain(SyntheticConfig {
            frames: 50,
            ..SyntheticConfig::default()
        });
        assert_eq!(frames.len(), 50);
        assert_eq!(frames[0].index, 0);
        assert_eq!(frames[49].index, 49);
    }

    #[test]
    fn is_deterministic_per_seed() {
        let a = drain(SyntheticConfig {
            seed: 7,
            ..SyntheticConfig::default()
        });
        let b = drain(SyntheticConfig {
            seed: 7,
            ..SyntheticConfig::default()
        });
        let c = drain(SyntheticConfig {
            seed: 8,
            ..SyntheticConfig::default()
        });
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn frame_times_stay_within_the_jitter_band_and_average_the_base() {
        let cfg = SyntheticConfig {
            base_frame_ms: 20.0,
            jitter_ms: 3.0,
            frames: 5000,
            ..SyntheticConfig::default()
        };
        let frames = drain(cfg);
        let deltas: Vec<f64> = frames
            .windows(2)
            .map(|w| w[1].render_ts_ms - w[0].render_ts_ms)
            .collect();
        assert!(
            deltas
                .iter()
                .all(|d| (17.0 - 1e-9..=23.0 + 1e-9).contains(d))
        );
        let mean = deltas.iter().sum::<f64>() / deltas.len() as f64;
        assert!((mean - 20.0).abs() < 0.2, "mean = {mean}");
    }

    #[test]
    fn timestamps_are_strictly_increasing() {
        let frames = drain(SyntheticConfig::default());
        assert!(
            frames
                .windows(2)
                .all(|w| w[1].render_ts_ms > w[0].render_ts_ms)
        );
    }

    #[test]
    fn user_marks_fire_once_at_the_requested_frames() {
        let mut p = SyntheticPlatform::new(SyntheticConfig {
            frames: 10,
            marks_at: vec![3],
            ..SyntheticConfig::default()
        });
        let mut marked = Vec::new();
        while let Some(frame) = p.next_frame().unwrap() {
            if p.poll_user_mark() {
                marked.push(frame.index);
            }
            assert!(!p.poll_user_mark(), "the mark is consumed");
        }
        assert_eq!(marked, [3]);
    }

    #[test]
    fn refresh_period_matches_the_rate_and_present_echoes_the_target() {
        let mut p = SyntheticPlatform::new(SyntheticConfig {
            refresh_hz: 144.0,
            ..SyntheticConfig::default()
        });
        assert!((p.refresh_period_ms() - 1000.0 / 144.0).abs() < 1e-9);
        let req = PresentRequest {
            source: FrameHandle(0),
            kind: sv_types::FrameKind::Rendered,
            target_ts_ms: 12.5,
        };
        assert!((p.present(&req).unwrap() - 12.5).abs() < f64::EPSILON);
    }
}
