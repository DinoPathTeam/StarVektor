//! Parser for the per-frame CSV written by `mangoapp` (MangoHud logging).
//!
//! Layout: an optional two-line system-information block (`os,cpu,gpu,...` and its
//! values), then a header (`fps,frametime,...,elapsed`), then one row per frame.
//! `frametime` is in milliseconds, `gpu_vram_used` in GB and `elapsed` in
//! nanoseconds since the start of the log.

use std::fmt;

/// One measured frame.
#[derive(Debug, Clone, PartialEq)]
pub struct MangoFrame {
    /// Time since the start of the log, in milliseconds.
    pub elapsed_ms: f64,
    /// Duration of the frame, in milliseconds.
    pub frametime_ms: f64,
    /// VRAM in use, in GB (as MangoHud reports it).
    pub vram_used_gb: f64,
}

/// Contents of a log.
#[derive(Debug, Clone, PartialEq)]
pub struct MangoLog {
    /// Kernel, driver, etc. from the system-information block, when present.
    pub system: Vec<(String, String)>,
    pub frames: Vec<MangoFrame>,
}

impl MangoLog {
    /// Value of a system-information field (`kernel`, `driver`, ...), if non-empty.
    #[must_use]
    pub fn system_field(&self, key: &str) -> Option<&str> {
        self.system
            .iter()
            .find(|(k, v)| k == key && !v.is_empty())
            .map(|(_, v)| v.as_str())
    }
}

impl MangoLog {
    /// Splits the log into consecutive windows of `window_ms` of log time.
    ///
    /// Each window is rebased to start at 0 ms. Windows whose frames span less
    /// than half a window (normally the tail of the recording) are dropped; the
    /// second value is how many were dropped.
    ///
    /// # Panics
    /// Never; a non-positive or non-finite window returns the whole log as one segment.
    #[must_use]
    pub fn split(&self, window_ms: f64) -> (Vec<MangoLog>, usize) {
        if !window_ms.is_finite() || window_ms <= 0.0 {
            return (vec![self.clone()], 0);
        }
        let mut segments: Vec<Vec<MangoFrame>> = Vec::new();
        let mut current_index: Option<u64> = None;
        for frame in &self.frames {
            // Truncation is intended: index of the window the frame falls in.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let index = (frame.elapsed_ms / window_ms) as u64;
            if current_index != Some(index) {
                current_index = Some(index);
                segments.push(Vec::new());
            }
            if let Some(last) = segments.last_mut() {
                last.push(frame.clone());
            }
        }
        let mut kept = Vec::new();
        let mut dropped = 0;
        for mut frames in segments {
            let start = frames.first().map_or(0.0, |f| f.elapsed_ms);
            let span = frames.last().map_or(0.0, |f| f.elapsed_ms) - start;
            if span < window_ms / 2.0 {
                dropped += 1;
                continue;
            }
            for frame in &mut frames {
                frame.elapsed_ms -= start;
            }
            kept.push(MangoLog {
                system: self.system.clone(),
                frames,
            });
        }
        (kept, dropped)
    }
}

/// Why a log could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// 1-based line number (0 when the problem is not tied to a line).
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.message)
        } else {
            write!(f, "line {}: {}", self.line, self.message)
        }
    }
}

impl std::error::Error for ParseError {}

fn err(line: usize, message: impl Into<String>) -> ParseError {
    ParseError {
        line,
        message: message.into(),
    }
}

/// Parses the text of a mangoapp CSV.
///
/// # Errors
/// If there is no header, a required column is missing, a row is malformed, the
/// clock goes backwards, or there are no frames.
pub fn parse(text: &str) -> Result<MangoLog, ParseError> {
    let mut lines = text.lines().enumerate().map(|(i, l)| (i + 1, l.trim()));
    let mut system_header: Option<Vec<&str>> = None;
    let mut system = Vec::new();

    // Find the frame header, remembering the system block if there is one.
    let (header_line, columns) = loop {
        let Some((no, line)) = lines.next() else {
            return Err(err(
                0,
                "no header found (expected a `fps,frametime,...` line)",
            ));
        };
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        if cols.first() == Some(&"fps") {
            break (no, cols);
        }
        match system_header.take() {
            None => system_header = Some(cols),
            Some(keys) => {
                system = keys
                    .iter()
                    .zip(cols.iter())
                    .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                    .collect();
            }
        }
    };

    let index = |name: &str| {
        columns
            .iter()
            .position(|c| *c == name)
            .ok_or_else(|| err(header_line, format!("missing column `{name}`")))
    };
    let (i_frametime, i_vram, i_elapsed) = (
        index("frametime")?,
        index("gpu_vram_used")?,
        index("elapsed")?,
    );

    let mut frames: Vec<MangoFrame> = Vec::new();
    for (no, line) in lines {
        if line.is_empty() {
            continue;
        }
        let cells: Vec<&str> = line.split(',').collect();
        if cells.len() != columns.len() {
            return Err(err(
                no,
                format!("expected {} columns, found {}", columns.len(), cells.len()),
            ));
        }
        let number = |i: usize, name: &str| -> Result<f64, ParseError> {
            cells[i]
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v >= 0.0)
                .ok_or_else(|| err(no, format!("`{name}` is not a valid number: {}", cells[i])))
        };
        let frame = MangoFrame {
            elapsed_ms: number(i_elapsed, "elapsed")? / 1_000_000.0,
            frametime_ms: number(i_frametime, "frametime")?,
            vram_used_gb: number(i_vram, "gpu_vram_used")?,
        };
        if let Some(previous) = frames.last().filter(|p| frame.elapsed_ms < p.elapsed_ms) {
            return Err(err(
                no,
                format!(
                    "elapsed goes backwards: {} ms after {} ms",
                    frame.elapsed_ms, previous.elapsed_ms
                ),
            ));
        }
        frames.push(frame);
    }

    if frames.is_empty() {
        return Err(err(0, "the log has a header but no frames"));
    }
    Ok(MangoLog { system, frames })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "fps,frametime,cpu_load,cpu_power,gpu_load,cpu_temp,gpu_temp,gpu_core_clock,gpu_mem_clock,gpu_vram_used,gpu_power,ram_used,swap_used,process_rss,cpu_mhz,elapsed";
    const SYSTEM: &str = "os,cpu,gpu,ram,kernel,driver,cpuscheduler\nCachyOS,AMD Ryzen 5 5500,,32721016,7.3.0-4-cachyos,,performance\n";

    fn row(frametime: f64, vram: f64, elapsed: u64) -> String {
        format!("100,{frametime},6,0,14,43,37,1561,1069,{vram},26,10,0.1,0,4441,{elapsed}\n")
    }

    fn sample() -> String {
        format!(
            "{SYSTEM}{HEADER}\n{}{}",
            row(9.98, 3.923, 19_612_432),
            row(10.0, 3.923, 29_621_529)
        )
    }

    #[test]
    fn reads_system_block_and_frames() {
        let log = parse(&sample()).unwrap();
        assert_eq!(log.frames.len(), 2);
        assert!((log.frames[0].elapsed_ms - 19.612_432).abs() < 1e-9);
        assert!((log.frames[1].frametime_ms - 10.0).abs() < 1e-9);
        assert!((log.frames[0].vram_used_gb - 3.923).abs() < 1e-9);
        assert_eq!(log.system_field("kernel"), Some("7.3.0-4-cachyos"));
        assert_eq!(log.system_field("driver"), None, "empty fields are absent");
    }

    #[test]
    fn works_without_the_system_block() {
        let text = format!("{HEADER}\n{}", row(10.0, 1.0, 1_000_000));
        let log = parse(&text).unwrap();
        assert!(log.system.is_empty());
        assert_eq!(log.frames.len(), 1);
    }

    #[test]
    fn rejects_missing_header_and_empty_logs() {
        assert!(parse("").is_err());
        assert!(parse("a,b\n1,2\n").is_err());
        let only_header = format!("{HEADER}\n");
        assert!(
            parse(&only_header)
                .unwrap_err()
                .message
                .contains("no frames")
        );
    }

    #[test]
    fn rejects_missing_columns() {
        let text = "fps,frametime,elapsed\n100,10,1000\n";
        assert!(parse(text).unwrap_err().message.contains("gpu_vram_used"));
    }

    #[test]
    fn rejects_malformed_rows_with_the_line_number() {
        let text = format!("{SYSTEM}{HEADER}\n{}1,2,3\n", row(10.0, 1.0, 1_000_000));
        assert_eq!(parse(&text).unwrap_err().line, 5);

        let bad = format!(
            "{HEADER}\n{}",
            row(10.0, 1.0, 1).replace("100,10", "100,abc")
        );
        let error = parse(&bad).unwrap_err();
        assert_eq!(error.line, 2);
        assert!(error.message.contains("frametime"));
    }

    #[test]
    fn rejects_a_clock_that_goes_backwards() {
        let text = format!(
            "{HEADER}\n{}{}",
            row(10.0, 1.0, 2_000_000),
            row(10.0, 1.0, 1_000_000)
        );
        assert!(parse(&text).unwrap_err().message.contains("backwards"));
    }

    #[test]
    fn splits_into_windows_rebased_to_zero_and_drops_a_short_tail() {
        // 10 ms frames for 7.5 s; 3 s windows -> [0,3) [3,6) kept, [6,7.5) = 1.5 s kept (half), none dropped.
        let mut text = format!("{HEADER}\n");
        for i in 1..=750_u64 {
            text.push_str(&row(10.0, 1.0, i * 10_000_000));
        }
        let log = parse(&text).unwrap();
        let (segments, dropped) = log.split(3000.0);
        assert_eq!(segments.len(), 3);
        assert_eq!(dropped, 0);
        assert!(
            segments[1].frames[0].elapsed_ms < 20.0,
            "rebased to the window start"
        );
        assert_eq!(segments.iter().map(|s| s.frames.len()).sum::<usize>(), 750);

        // A tail under half a window is dropped.
        let (segments, dropped) = log.split(6000.0);
        assert_eq!((segments.len(), dropped), (1, 1));

        // Degenerate windows keep the whole log.
        assert_eq!(log.split(0.0).0.len(), 1);
        assert_eq!(log.split(f64::NAN).0.len(), 1);
    }

    #[test]
    fn rejects_negative_or_non_finite_numbers() {
        let text = format!("{HEADER}\n{}", row(-1.0, 1.0, 1_000_000));
        assert!(parse(&text).is_err());
    }
}
