//! `starvektor measure`: turns a mangoapp CSV into a validated run log (H1.1).
//!
//! This is the Phase 1 baseline: **nothing is generated**. Every frame is logged as
//! rendered, at level 0, with no pass costs and no invented latency. The data comes
//! from files written by other tools (mangoapp, amdgpu sysfs) outside the game
//! process; no frame images are read or stored.
//!
//! Because nothing is generated, rendered and presented times are the same here:
//! mangoapp reports what gamescope presented.

use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sv_config::io::{default_log_dir, process_env};
use sv_platform_linux::mangoapp::{self, MangoLog};
use sv_platform_linux::sysfs::AmdgpuSysfs;
use sv_telemetry::schema::{FrameRecord, Record, RunRecord, SummaryRecord};
use sv_telemetry::summary::SummaryBuilder;
use sv_telemetry::time::utc_now_iso8601;
use sv_telemetry::validate::{ValidationReport, validate};
use sv_telemetry::writer::LogWriter;
use sv_types::{FrameKind, IntegrationLevel, Level, OptiScalerMode, PassTimings, Resolution};

use super::CommandResult;
use crate::cli::MeasureArgs;
use crate::ids;

/// Labels and context of a measured run.
#[derive(Debug, Clone)]
pub struct MeasureOptions {
    pub game: String,
    pub scene: String,
    pub gpu: String,
    pub resolution: (u32, u32),
    /// Total VRAM in MiB, to derive the free VRAM. `None` if it could not be read.
    pub vram_total_mb: Option<u32>,
}

/// What the measurement produced.
#[derive(Debug)]
pub struct MeasureOutcome {
    pub out: PathBuf,
    pub frames: u64,
    pub summary: Option<SummaryRecord>,
    pub validation: ValidationReport,
}

pub fn run(args: &MeasureArgs) -> CommandResult {
    let text = fs::read_to_string(&args.csv)
        .map_err(|e| format!("cannot read {}: {e}", args.csv.display()))?;

    let sysfs = match &args.sysfs {
        Some(dir) => Some(AmdgpuSysfs::new(dir)),
        None => AmdgpuSysfs::detect(Path::new("/sys/class/drm")),
    };
    let vram_total_mb = sysfs.and_then(|s| s.total_mb().ok());
    if vram_total_mb.is_none() {
        eprintln!("warning: total VRAM unavailable; vram_free_mb will be 0 in this log");
    }

    let run_id = ids::new_run_id();
    let out = args.out.clone().unwrap_or_else(|| {
        default_log_dir(&process_env)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(format!("measure-{}.jsonl", &run_id[..8]))
    });
    let options = MeasureOptions {
        game: args.game.clone(),
        scene: args.scene.clone(),
        gpu: args.gpu.clone(),
        resolution: args.resolution,
        vram_total_mb,
    };
    let outcome = measure_text(&text, &options, &out, &run_id)?;

    println!(
        "measure ok: {} frames -> {}",
        outcome.frames,
        outcome.out.display()
    );
    if let Some(s) = &outcome.summary {
        println!(
            "  fps avg {:.1} | p1 {:.2} ms | p0.1 {:.2} ms | no frame generation (baseline)",
            s.fps_presented_avg, s.p1_ms, s.p01_ms
        );
    }
    println!(
        "  log validated: {} records, {} frames",
        outcome.validation.records, outcome.validation.frames
    );
    Ok(ExitCode::SUCCESS)
}

/// Builds and validates the log from the text of a mangoapp CSV.
///
/// # Errors
/// If the CSV is invalid, the log cannot be written, or it fails validation.
pub fn measure_text(
    csv: &str,
    options: &MeasureOptions,
    out: &Path,
    run_id: &str,
) -> Result<MeasureOutcome, Box<dyn Error>> {
    let log = mangoapp::parse(csv)?;
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }

    let run_record = RunRecord {
        run_id: run_id.to_owned(),
        started_at: utc_now_iso8601(),
        game: options.game.clone(),
        game_version: env!("CARGO_PKG_VERSION").to_owned(),
        scene: options.scene.clone(),
        gpu: options.gpu.clone(),
        driver: driver_label(&log),
        mesa: None,
        resolution: Resolution(options.resolution.0, options.resolution.1),
        optiscaler: OptiScalerMode::None,
        integration_level: IntegrationLevel::ImageOnly,
        model_id: None,
        model_hash: None,
        seed: 0,
        config: serde_json::json!({
            "source": "mangoapp",
            "generation": false,
            "vram_total_mb": options.vram_total_mb,
        }),
    };
    let capacity = log.frames.len().saturating_add(64).min(1 << 22);
    let writer = LogWriter::create(out, run_record, capacity)?;

    let mut summary = SummaryBuilder::new();
    for (n, frame) in (0_u64..).zip(&log.frames) {
        let used_mb = gb_to_mb(frame.vram_used_gb);
        let record = FrameRecord {
            n,
            t_render_ms: frame.elapsed_ms,
            t_present_ms: frame.elapsed_ms,
            kind: FrameKind::Rendered,
            pass_ms: PassTimings::default(),
            vram_used_mb: used_mb,
            vram_free_mb: options
                .vram_total_mb
                .map_or(0, |total| total.saturating_sub(used_mb)),
            level: Level::Off,
            repeat_pct: 0.0,
            ssim: None,
            psnr: None,
        };
        summary.push(&record);
        writer.log(Record::Frame(record));
    }
    let frames = summary.frames();
    // No latency estimate: nothing here measures it.
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
    Ok(MeasureOutcome {
        out: out.to_path_buf(),
        frames,
        summary: summary_record,
        validation,
    })
}

fn driver_label(log: &MangoLog) -> String {
    let mut parts = vec!["mangoapp".to_owned()];
    if let Some(kernel) = log.system_field("kernel") {
        parts.push(format!("kernel {kernel}"));
    }
    if let Some(driver) = log.system_field("driver") {
        parts.push(driver.to_owned());
    }
    parts.join(" / ")
}

/// GB (as MangoHud reports) to MiB, saturating.
fn gb_to_mb(gb: f64) -> u32 {
    let mb = (gb * 1024.0).round();
    if mb >= f64::from(u32::MAX) {
        u32::MAX
    } else {
        // Non-negative and in range by the checks above.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            mb as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "fps,frametime,cpu_load,cpu_power,gpu_load,cpu_temp,gpu_temp,gpu_core_clock,gpu_mem_clock,gpu_vram_used,gpu_power,ram_used,swap_used,process_rss,cpu_mhz,elapsed";

    fn csv(frames: usize) -> String {
        let mut text = String::from(
            "os,cpu,gpu,ram,kernel,driver,cpuscheduler\nCachyOS,cpu,,1,7.3.0,,performance\n",
        );
        text.push_str(HEADER);
        text.push('\n');
        for i in 0..frames {
            let elapsed = 10_000_000_u64 * (i as u64 + 1);
            text.push_str(&format!(
                "100,10,6,0,14,43,37,1561,1069,3.5,26,10,0.1,0,4441,{elapsed}\n"
            ));
        }
        text
    }

    fn options(total: Option<u32>) -> MeasureOptions {
        MeasureOptions {
            game: "test".into(),
            scene: "unit".into(),
            gpu: "gpu".into(),
            resolution: (1920, 1080),
            vram_total_mb: total,
        }
    }

    fn temp_out(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("sv-measure-{}-{name}.jsonl", std::process::id()))
    }

    #[test]
    fn builds_a_valid_baseline_log() {
        let out = temp_out("ok");
        let outcome = measure_text(&csv(50), &options(Some(8192)), &out, "run-1").unwrap();
        assert_eq!(outcome.frames, 50);
        assert_eq!(outcome.validation.frames, 50);
        let summary = outcome.summary.unwrap();
        assert!(
            (summary.fps_presented_avg - 100.0).abs() < 1.0,
            "{summary:?}"
        );
        assert!(summary.latency_est_ms.is_none());
        let text = fs::read_to_string(&out).unwrap();
        assert!(text.contains("\"vram_used_mb\":3584"), "3.5 GB -> 3584 MiB");
        assert!(text.contains("\"vram_free_mb\":4608"));
        assert!(text.contains("\"generation\":false"));
        fs::remove_file(out).unwrap();
    }

    #[test]
    fn free_vram_is_zero_without_a_total() {
        let out = temp_out("nototal");
        measure_text(&csv(5), &options(None), &out, "run-2").unwrap();
        assert!(
            fs::read_to_string(&out)
                .unwrap()
                .contains("\"vram_free_mb\":0")
        );
        fs::remove_file(out).unwrap();
    }

    #[test]
    fn rejects_an_invalid_csv() {
        let out = temp_out("bad");
        assert!(measure_text("garbage", &options(None), &out, "run-3").is_err());
        assert!(!out.exists(), "no log is created for an invalid CSV");
    }

    #[test]
    fn converts_gb_to_mib() {
        assert_eq!(gb_to_mb(3.923), 4017);
        assert_eq!(gb_to_mb(0.0), 0);
        assert_eq!(gb_to_mb(1e12), u32::MAX);
    }
}
