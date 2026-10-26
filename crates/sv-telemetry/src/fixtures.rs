//! Datos de ejemplo compartidos por las pruebas de este crate.

use std::collections::BTreeMap;

use sv_types::{FrameKind, IntegrationLevel, Level, OptiScalerMode, PassTimings, Resolution};

use crate::schema::{FrameRecord, RunRecord, SummaryRecord};

pub fn run() -> RunRecord {
    RunRecord {
        run_id: "00000000-0000-4000-8000-000000000001".into(),
        started_at: "2026-10-26T17:00:00Z".into(),
        game: "juego-de-prueba".into(),
        game_version: "1.0".into(),
        scene: "exploracion".into(),
        gpu: "RX 9060 XT".into(),
        driver: "mesa".into(),
        mesa: Some("26.0".into()),
        resolution: Resolution(2560, 1440),
        optiscaler: OptiScalerMode::Upscale,
        integration_level: IntegrationLevel::ImageOnly,
        model_id: None,
        model_hash: None,
        seed: 42,
        config: serde_json::json!({}),
    }
}

pub fn frame(n: u64, t_present_ms: f64) -> FrameRecord {
    FrameRecord {
        n,
        t_render_ms: t_present_ms - 1.0,
        t_present_ms,
        kind: FrameKind::Rendered,
        pass_ms: PassTimings {
            capture: 0.3,
            motion: 1.0,
            reproject: 0.5,
            mask: 0.4,
            present: 0.2,
        },
        vram_used_mb: 3000,
        vram_free_mb: 5000,
        level: Level::Classic,
        repeat_pct: 0.0,
        ssim: None,
        psnr: None,
    }
}

pub fn summary() -> SummaryRecord {
    let mut level_time_pct = BTreeMap::new();
    for key in ["0", "1", "2", "3"] {
        level_time_pct.insert(key.to_owned(), 25.0);
    }
    SummaryRecord {
        fps_rendered_avg: 60.0,
        fps_presented_avg: 60.0,
        p1_ms: 18.0,
        p01_ms: 20.0,
        latency_est_ms: Some(25.0),
        overhead_ms_avg: 2.4,
        level_time_pct,
    }
}
