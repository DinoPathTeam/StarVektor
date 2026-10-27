//! Log record types. Source of truth for the schema.
//!
//! Evolution rules (see `docs/log-schema.md`):
//! - Adding an optional field is compatible: it bumps the minor version (`1.1`).
//! - Removing, renaming or changing the type of a field is incompatible: it bumps the major.
//! - Readers accept any minor version of the major they know.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sv_types::{FrameKind, IntegrationLevel, Level, OptiScalerMode, PassTimings, Resolution};

/// Schema version written by this version of StarVektor.
pub const SCHEMA_VERSION: &str = "1.0";

/// Major version the validator can read.
pub const SUPPORTED_MAJOR: &str = "1";

/// One line of the file: the schema version plus the record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub schema: String,
    #[serde(flatten)]
    pub record: Record,
}

impl Line {
    #[must_use]
    pub fn new(record: Record) -> Self {
        Self {
            schema: SCHEMA_VERSION.to_owned(),
            record,
        }
    }
}

/// Log record. The `type` field distinguishes it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Record {
    /// Boxed: it appears once per file and is much larger than `Frame`,
    /// which is the one that travels through the write queue on every frame.
    Run(Box<RunRecord>),
    Frame(FrameRecord),
    Event(EventRecord),
    Summary(SummaryRecord),
}

/// Once, at the start. Must be the first line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    /// ISO 8601 in UTC, for example `2026-10-26T17:00:00Z`.
    pub started_at: String,
    pub game: String,
    pub game_version: String,
    /// Fixed test scene (exploration, combat, changing light).
    pub scene: String,
    pub gpu: String,
    pub driver: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesa: Option<String>,
    pub resolution: Resolution,
    pub optiscaler: OptiScalerMode,
    pub integration_level: IntegrationLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_hash: Option<String>,
    /// Seed to repeat the exact test.
    pub seed: u64,
    /// Full effective configuration (copy of the resolved TOML).
    pub config: serde_json::Value,
}

/// One per presented frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameRecord {
    /// Index of the presented frame, strictly increasing.
    pub n: u64,
    pub t_render_ms: f64,
    pub t_present_ms: f64,
    pub kind: FrameKind,
    pub pass_ms: PassTimings,
    pub vram_used_mb: u32,
    pub vram_free_mb: u32,
    pub level: Level,
    /// Percentage of the screen where the mask fell back to repeating (0 to 100).
    pub repeat_pct: f32,
    /// Only if ground truth is available (0 to 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssim: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub psnr: Option<f32>,
}

/// Type of point event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    LevelChange,
    SceneCut,
    FrameSpike,
    VramLow,
    /// Manual mark by the user: "I saw an artifact here".
    UserMark,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    /// Associated frame.
    pub n: u64,
    pub event: EventKind,
    pub detail: serde_json::Value,
}

/// One per scene, on close. Must be the last line.
///
/// `p1_ms` and `p01_ms` are the frame times at the 99th and 99.9th percentile
/// (the slowest 1 % and 0.1 %). They are computed with the nearest-rank method
/// over the intervals between presented frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryRecord {
    pub fps_rendered_avg: f64,
    pub fps_presented_avg: f64,
    pub p1_ms: f64,
    pub p01_ms: f64,
    /// Estimated added latency, in ms. Absent if it could not be estimated
    /// (for example, in the synthetic `selftest`): never filled with a fake 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_est_ms: Option<f64>,
    pub overhead_ms_avg: f64,
    /// Percentage of presented frames at each level; keys `"0"` a `"3"`.
    pub level_time_pct: BTreeMap<String, f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn line_roundtrips_for_every_record_type() {
        let records = [
            Record::Run(Box::new(fixtures::run())),
            Record::Frame(fixtures::frame(7, 100.0)),
            Record::Event(EventRecord {
                n: 7,
                event: EventKind::UserMark,
                detail: serde_json::json!({"note": "halo on the HUD"}),
            }),
            Record::Summary(fixtures::summary()),
        ];
        for record in records {
            let line = Line::new(record);
            let text = serde_json::to_string(&line).unwrap();
            let back: Line = serde_json::from_str(&text).unwrap();
            assert_eq!(back, line, "{text}");
        }
    }

    #[test]
    fn wire_format_matches_the_documented_shape() {
        let line = Line::new(Record::Frame(fixtures::frame(3, 50.0)));
        let value: serde_json::Value = serde_json::to_value(&line).unwrap();
        assert_eq!(value["schema"], "1.0");
        assert_eq!(value["type"], "frame");
        assert_eq!(value["n"], 3);
        assert_eq!(value["kind"], "rendered");
        assert_eq!(value["level"], 1, "the level is written as an integer");
        assert!(value["pass_ms"]["motion"].is_number());
        assert!(
            value.get("ssim").is_none(),
            "empty optionals are not written"
        );
    }

    #[test]
    fn run_resolution_is_an_array() {
        let line = Line::new(Record::Run(Box::new(fixtures::run())));
        let value = serde_json::to_value(&line).unwrap();
        assert_eq!(value["resolution"], serde_json::json!([2560, 1440]));
        assert_eq!(value["optiscaler"], "upscale");
        assert_eq!(value["integration_level"], "image_only");
    }
}
