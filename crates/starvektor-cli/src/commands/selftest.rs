//! `starvektor selftest`: runs the whole system against a synthetic game.
//!
//! It checks the **connection** between layers (platform -> predictor -> regulator
//! -> telemetry -> validator), not performance: the game's timings and the
//! cost per level are made up. Real figures come from Phase 1.
//! It meets the Phase 0 exit criterion: a valid `run` written and
//! checked against the schema.

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sv_config::Config;
use sv_config::io::{default_log_dir, process_env};
use sv_core::regulator::{Reason, Regulator, Sample, Transition};
use sv_core::timing::FrameTimePredictor;
use sv_gfx::{GraphicsBackend, NullBackend};
use sv_platform::Platform;
use sv_platform::synthetic::{SyntheticConfig, SyntheticPlatform};
use sv_telemetry::schema::{EventKind, EventRecord, FrameRecord, Record, RunRecord, SummaryRecord};
use sv_telemetry::summary::SummaryBuilder;
use sv_telemetry::time::utc_now_iso8601;
use sv_telemetry::validate::{ValidationReport, validate};
use sv_telemetry::writer::{LogWriter, WriteStats};
use sv_types::{FrameKind, IntegrationLevel, Level, MemoryBudget, OptiScalerMode, PassTimings};

use super::{CommandResult, config_path};
use crate::cli::SelftestArgs;
use crate::{ids, wiring};

/// Name under which the selftest looks up its profile in the configuration.
pub const GAME_NAME: &str = "selftest";

#[derive(Debug, Clone, Copy)]
pub struct SelftestOptions {
    pub frames: u64,
    pub seed: u64,
}

/// What happened during the run.
#[derive(Debug)]
pub struct SelftestOutcome {
    pub out: PathBuf,
    /// Level changes and the frame at which they occurred.
    pub transitions: Vec<(u64, Transition)>,
    pub final_level: Level,
    pub summary: Option<SummaryRecord>,
    pub frames_logged: u64,
    pub stats: WriteStats,
    pub validation: ValidationReport,
}

pub fn run(args: &SelftestArgs, explicit_config: Option<&Path>) -> CommandResult {
    let path = config_path(explicit_config)?;
    let mut config = Config::load_or_default(&path)?;
    config.apply_env(&process_env)?;

    let run_id = ids::new_run_id();
    let out = args.out.clone().unwrap_or_else(|| {
        default_log_dir(&process_env)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(format!("selftest-{}.jsonl", &run_id[..8]))
    });

    let outcome = run_selftest(
        &config,
        SelftestOptions {
            frames: args.frames,
            seed: args.seed,
        },
        &out,
        &run_id,
    )?;
    print_outcome(&outcome);
    Ok(ExitCode::SUCCESS)
}

/// Runs the selftest and validates the resulting log.
///
/// # Errors
/// If the platform, the backend, the writing or the log validation fails.
pub fn run_selftest(
    config: &Config,
    options: SelftestOptions,
    out: &Path,
    run_id: &str,
) -> Result<SelftestOutcome, Box<dyn Error>> {
    let effective = config.effective(GAME_NAME);
    let mut regulator = Regulator::new(wiring::regulator_config(config, &effective));
    let mut predictor = FrameTimePredictor::new(0.1);
    let synthetic = SyntheticConfig {
        frames: options.frames,
        seed: options.seed,
        marks_at: vec![options.frames / 2],
        ..SyntheticConfig::default()
    };
    let resolution = synthetic.resolution;
    let mut platform = SyntheticPlatform::new(synthetic);
    let refresh_ms = platform.refresh_period_ms();
    let mut gfx = NullBackend::new(
        MemoryBudget {
            used_mb: 3000,
            free_mb: 5000,
        },
        PassTimings::default(),
    );

    let run_record = RunRecord {
        run_id: run_id.to_owned(),
        started_at: utc_now_iso8601(),
        game: GAME_NAME.to_owned(),
        game_version: env!("CARGO_PKG_VERSION").to_owned(),
        scene: "synthetic".to_owned(),
        gpu: "none (synthetic)".to_owned(),
        driver: format!("{}/{}", platform.name(), gfx.name()),
        mesa: None,
        resolution,
        optiscaler: OptiScalerMode::None,
        integration_level: IntegrationLevel::ImageOnly,
        model_id: None,
        model_hash: None,
        seed: options.seed,
        config: serde_json::to_value(config)?,
    };
    let capacity = usize::try_from(
        options
            .frames
            .saturating_mul(4)
            .saturating_add(64)
            .min(1 << 20),
    )
    .unwrap_or(1 << 20);
    let writer = LogWriter::create(out, run_record, capacity)?;

    let mut summary = SummaryBuilder::new();
    let mut transitions = Vec::new();
    let mut n = 0_u64;
    let mut previous_render_ms: Option<f64> = None;

    while let Some(captured) = platform.next_frame()? {
        if let Some(previous) = previous_render_ms {
            predictor.observe(captured.render_ts_ms - previous);
        }
        previous_render_ms = Some(captured.render_ts_ms);
        let base_ms = predictor.predict_ms().unwrap_or(refresh_ms);

        // The cost depends on the level at which this frame is processed.
        let level = regulator.level();
        gfx.set_timings(synthetic_cost(level));
        let memory = gfx.memory_budget()?;
        let timings = gfx.last_pass_timings()?;
        let overhead_ms = f64::from(timings.total());
        let transition = regulator.observe(&Sample {
            base_frame_ms: base_ms,
            refresh_ms,
            overhead_ms,
            vram_free_mb: memory.free_mb,
        });

        let frame_n = n;
        let present_ms = captured.render_ts_ms + overhead_ms;
        let template = FrameRecord {
            n,
            t_render_ms: captured.render_ts_ms,
            t_present_ms: present_ms,
            kind: FrameKind::Rendered,
            pass_ms: timings,
            vram_used_mb: memory.used_mb,
            vram_free_mb: memory.free_mb,
            level,
            repeat_pct: 0.0,
            ssim: None,
            psnr: None,
        };
        summary.push(&template);
        writer.log(Record::Frame(template.clone()));
        n += 1;

        if level >= Level::Classic {
            // Frame generated halfway to the next expected one.
            let generated = FrameRecord {
                n,
                t_present_ms: present_ms + base_ms / 2.0,
                kind: FrameKind::Generated,
                ..template
            };
            summary.push(&generated);
            writer.log(Record::Frame(generated));
            n += 1;
        }

        if let Some(t) = transition {
            transitions.push((frame_n, t));
            writer.log(Record::Event(EventRecord {
                n: frame_n,
                event: EventKind::LevelChange,
                detail: serde_json::json!({
                    "from": u8::from(t.from),
                    "to": u8::from(t.to),
                    "reason": t.reason.as_str(),
                }),
            }));
            if t.reason == Reason::VramLow {
                writer.log(Record::Event(EventRecord {
                    n: frame_n,
                    event: EventKind::VramLow,
                    detail: serde_json::json!({ "vram_free_mb": memory.free_mb }),
                }));
            }
        }
        if platform.poll_user_mark() {
            writer.log(Record::Event(EventRecord {
                n: frame_n,
                event: EventKind::UserMark,
                detail: serde_json::json!({ "source": platform.name() }),
            }));
        }
    }

    let frames_logged = summary.frames();
    // No estimated latency: the selftest measures none, and the log must not invent it.
    let summary_record = summary.finish(None);
    if let Some(record) = &summary_record {
        writer.log(Record::Summary(record.clone()));
    }
    let stats = writer.finish()?;
    if stats.dropped > 0 {
        return Err(format!("{} log records were dropped", stats.dropped).into());
    }

    let validation = validate(BufReader::new(File::open(out)?)).map_err(|issues| {
        let lines: Vec<String> = issues.iter().map(ToString::to_string).collect();
        format!(
            "the generated log does not comply with the schema: {}",
            lines.join("; ")
        )
    })?;

    Ok(SelftestOutcome {
        out: out.to_path_buf(),
        transitions,
        final_level: regulator.level(),
        summary: summary_record,
        frames_logged,
        stats,
        validation,
    })
}

/// Made-up cost per level, only to exercise the regulator.
fn synthetic_cost(level: Level) -> PassTimings {
    match level {
        Level::Off => PassTimings::default(),
        Level::Classic => PassTimings {
            capture: 0.2,
            motion: 0.6,
            reproject: 0.3,
            mask: 0.2,
            present: 0.1,
        },
        Level::MiniNet => PassTimings {
            capture: 0.2,
            motion: 0.6,
            reproject: 0.3,
            mask: 0.7,
            present: 0.2,
        },
        Level::FullNet => PassTimings {
            capture: 0.2,
            motion: 0.9,
            reproject: 0.4,
            mask: 0.7,
            present: 0.2,
        },
    }
}

fn print_outcome(outcome: &SelftestOutcome) {
    println!("selftest ok");
    println!("  log: {}", outcome.out.display());
    println!(
        "  records: {} ({} presented frames, {} events)",
        outcome.validation.records, outcome.validation.frames, outcome.validation.events
    );
    if outcome.transitions.is_empty() {
        println!(
            "  level changes: none (final level {})",
            u8::from(outcome.final_level)
        );
    } else {
        for (frame, t) in &outcome.transitions {
            println!(
                "  frame {frame}: level {} -> {} ({})",
                u8::from(t.from),
                u8::from(t.to),
                t.reason.as_str()
            );
        }
    }
    if let Some(s) = &outcome.summary {
        println!(
            "  summary: {:.1} rendered fps, {:.1} presented fps, p1 {:.1} ms, mean cost {:.2} ms",
            s.fps_rendered_avg, s.fps_presented_avg, s.p1_ms, s.overhead_ms_avg
        );
    }
    println!(
        "  note: synthetic timings and costs; this tests the connection between layers, not performance."
    );
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use sv_config::model::Profile;

    use super::*;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_file(name: &str) -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sv-selftest-{}-{id}-{name}", std::process::id()))
    }

    fn run_with(config: &Config, frames: u64, seed: u64) -> SelftestOutcome {
        let out = temp_file("log.jsonl");
        let outcome =
            run_selftest(config, SelftestOptions { frames, seed }, &out, "test-run").unwrap();
        std::fs::remove_file(&out).unwrap();
        outcome
    }

    #[test]
    fn a_slow_game_wakes_the_regulator_and_climbs_with_headroom() {
        let outcome = run_with(&Config::default(), 600, 1);
        let levels: Vec<(Level, Level, Reason)> = outcome
            .transitions
            .iter()
            .map(|(_, t)| (t.from, t.to, t.reason))
            .collect();
        assert_eq!(
            levels,
            [
                (Level::Off, Level::Classic, Reason::Needed),
                (Level::Classic, Level::MiniNet, Reason::Headroom)
            ]
        );
        assert_eq!(
            outcome.final_level,
            Level::MiniNet,
            "MiniNet costs 2.0 ms: no headroom for FullNet"
        );
    }

    #[test]
    fn generation_raises_presented_fps_above_rendered_fps() {
        let outcome = run_with(&Config::default(), 600, 1);
        let s = outcome.summary.unwrap();
        assert!(
            (s.fps_rendered_avg - 1000.0 / 22.0).abs() < 2.0,
            "{}",
            s.fps_rendered_avg
        );
        assert!(s.fps_presented_avg > s.fps_rendered_avg * 1.4, "{s:?}");
        assert!(s.latency_est_ms.is_none(), "no latency is invented");
        assert!(s.overhead_ms_avg > 0.0);
        assert_eq!(outcome.frames_logged, outcome.validation.frames);
    }

    #[test]
    fn the_log_contains_the_user_mark_and_level_events() {
        let out = temp_file("events.jsonl");
        let outcome = run_selftest(
            &Config::default(),
            SelftestOptions {
                frames: 400,
                seed: 3,
            },
            &out,
            "id",
        )
        .unwrap();
        let text = std::fs::read_to_string(&out).unwrap();
        std::fs::remove_file(&out).unwrap();
        assert_eq!(
            text.lines()
                .filter(|l| l.contains("\"event\":\"user_mark\""))
                .count(),
            1
        );
        assert_eq!(
            text.lines()
                .filter(|l| l.contains("\"event\":\"level_change\""))
                .count(),
            outcome.transitions.len()
        );
        assert!(text.lines().next().unwrap().contains("\"run_id\":\"id\""));
        assert!(outcome.validation.has_summary);
    }

    #[test]
    fn same_seed_same_behaviour() {
        let a = run_with(&Config::default(), 500, 9);
        let b = run_with(&Config::default(), 500, 9);
        let frames = |o: &SelftestOutcome| {
            o.transitions
                .iter()
                .map(|(f, t)| (*f, t.to))
                .collect::<Vec<_>>()
        };
        assert_eq!(frames(&a), frames(&b));
        assert_eq!(a.frames_logged, b.frames_logged);
    }

    #[test]
    fn a_zero_cap_never_generates() {
        let mut config = Config::default();
        config.regulator.max_level = Level::Off;
        let outcome = run_with(&config, 300, 1);
        assert!(outcome.transitions.is_empty());
        assert_eq!(outcome.final_level, Level::Off);
        assert_eq!(
            outcome.frames_logged, 300,
            "no generation: one presented frame per rendered frame"
        );
    }

    #[test]
    fn a_disabled_profile_behaves_like_a_zero_cap() {
        let mut config = Config::default();
        config.profiles.insert(
            GAME_NAME.into(),
            Profile {
                enabled: Some(false),
                ..Profile::default()
            },
        );
        let outcome = run_with(&config, 300, 1);
        assert_eq!(outcome.final_level, Level::Off);
        assert_eq!(outcome.frames_logged, 300);
    }

    #[test]
    fn zero_frames_still_produces_a_valid_run_only_log() {
        let outcome = run_with(&Config::default(), 0, 1);
        assert_eq!(outcome.validation.records, 1);
        assert!(outcome.summary.is_none());
    }

    #[test]
    fn an_unwritable_destination_is_a_clean_error() {
        let dir = std::env::temp_dir();
        let err = run_selftest(
            &Config::default(),
            SelftestOptions { frames: 5, seed: 1 },
            &dir,
            "id",
        )
        .unwrap_err();
        assert!(!err.to_string().is_empty());
    }
}
