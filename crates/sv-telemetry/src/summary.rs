//! Computation of the `summary` record from the `frame` records.

use std::collections::BTreeMap;

use sv_types::FrameKind;

use crate::schema::{FrameRecord, SummaryRecord};

/// Accumulates frames and produces the scene summary.
#[derive(Debug, Default)]
pub struct SummaryBuilder {
    frames: u64,
    rendered: u64,
    first_present_ms: Option<f64>,
    last_present_ms: f64,
    intervals_ms: Vec<f64>,
    overhead_sum_ms: f64,
    level_counts: [u64; 4],
}

impl SummaryBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a presented frame, in order.
    pub fn push(&mut self, frame: &FrameRecord) {
        match self.first_present_ms {
            None => self.first_present_ms = Some(frame.t_present_ms),
            Some(_) => self
                .intervals_ms
                .push(frame.t_present_ms - self.last_present_ms),
        }
        self.last_present_ms = frame.t_present_ms;
        self.frames += 1;
        if frame.kind == FrameKind::Rendered {
            self.rendered += 1;
        }
        self.overhead_sum_ms += f64::from(frame.pass_ms.total());
        self.level_counts[usize::from(u8::from(frame.level))] += 1;
    }

    /// Frames accumulated so far.
    #[must_use]
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Closes the summary. `None` if there were no frames.
    ///
    /// `latency_est_ms` is supplied by whoever estimates it (it does not come from
    /// the frames); `None` if there is no estimate.
    #[must_use]
    pub fn finish(mut self, latency_est_ms: Option<f64>) -> Option<SummaryRecord> {
        let first = self.first_present_ms?;
        let duration_s = (self.last_present_ms - first) / 1000.0;
        let rate = |count: u64| {
            if duration_s > 0.0 {
                count.saturating_sub(1) as f64 / duration_s
            } else {
                0.0
            }
        };

        self.intervals_ms.sort_by(f64::total_cmp);
        let total = self.frames as f64;
        let level_time_pct: BTreeMap<String, f64> = self
            .level_counts
            .iter()
            .enumerate()
            .map(|(level, count)| (level.to_string(), *count as f64 / total * 100.0))
            .collect();

        Some(SummaryRecord {
            fps_rendered_avg: rate(self.rendered),
            fps_presented_avg: rate(self.frames),
            p1_ms: percentile(&self.intervals_ms, 0.99),
            p01_ms: percentile(&self.intervals_ms, 0.999),
            latency_est_ms,
            overhead_ms_avg: self.overhead_sum_ms / total,
            level_time_pct,
        })
    }
}

/// Nearest-rank percentile over already sorted data. 0 if there is no data.
fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (q * sorted.len() as f64).ceil().max(1.0) as usize;
    sorted[rank.min(sorted.len()) - 1]
}

#[cfg(test)]
mod tests {
    use sv_types::Level;

    use super::*;
    use crate::fixtures;

    #[test]
    fn no_frames_gives_no_summary() {
        assert!(SummaryBuilder::new().finish(None).is_none());
    }

    #[test]
    fn steady_60_fps_summary() {
        let mut b = SummaryBuilder::new();
        for i in 0..601 {
            b.push(&fixtures::frame(i, i as f64 * 1000.0 / 60.0));
        }
        let s = b.finish(Some(25.0)).unwrap();
        assert!(
            (s.fps_presented_avg - 60.0).abs() < 1e-6,
            "{}",
            s.fps_presented_avg
        );
        assert!((s.fps_rendered_avg - 60.0).abs() < 1e-6);
        assert!((s.p1_ms - 1000.0 / 60.0).abs() < 1e-6);
        assert!((s.overhead_ms_avg - 2.4).abs() < 1e-5);
        assert_eq!(s.latency_est_ms, Some(25.0));
    }

    #[test]
    fn percentiles_pick_the_slow_tail() {
        // 1000 intervals from 1 to 1000 ms.
        let mut b = SummaryBuilder::new();
        let mut t = 0.0;
        b.push(&fixtures::frame(0, t));
        for i in 1..=1000_u32 {
            t += f64::from(i);
            b.push(&fixtures::frame(u64::from(i), t));
        }
        let s = b.finish(None).unwrap();
        assert!((s.p1_ms - 990.0).abs() < 1e-9, "p1 = {}", s.p1_ms);
        assert!((s.p01_ms - 999.0).abs() < 1e-9, "p01 = {}", s.p01_ms);
    }

    #[test]
    fn generated_frames_raise_presented_but_not_rendered_fps() {
        let mut b = SummaryBuilder::new();
        for i in 0..201u64 {
            let mut f = fixtures::frame(i, i as f64 * 10.0);
            if i % 2 == 1 {
                f.kind = FrameKind::Generated;
            }
            b.push(&f);
        }
        let s = b.finish(None).unwrap();
        assert!((s.fps_presented_avg - 100.0).abs() < 1e-6);
        // 101 rendered in 2 s: (101 - 1) / 2 = 50 fps.
        assert!(
            (s.fps_rendered_avg - 50.0).abs() < 1e-6,
            "{}",
            s.fps_rendered_avg
        );
    }

    #[test]
    fn level_share_sums_to_100_and_has_all_keys() {
        let mut b = SummaryBuilder::new();
        for i in 0..100u64 {
            let mut f = fixtures::frame(i, i as f64 * 16.0);
            f.level = if i < 25 { Level::Off } else { Level::MiniNet };
            b.push(&f);
        }
        let s = b.finish(None).unwrap();
        assert_eq!(s.level_time_pct.len(), 4);
        assert!((s.level_time_pct["0"] - 25.0).abs() < 1e-9);
        assert!((s.level_time_pct["2"] - 75.0).abs() < 1e-9);
        assert!((s.level_time_pct.values().sum::<f64>() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn single_frame_has_zero_rates() {
        let mut b = SummaryBuilder::new();
        b.push(&fixtures::frame(0, 5.0));
        let s = b.finish(None).unwrap();
        assert!(s.fps_presented_avg.abs() < f64::EPSILON);
        assert!(s.p1_ms.abs() < f64::EPSILON);
    }
}
