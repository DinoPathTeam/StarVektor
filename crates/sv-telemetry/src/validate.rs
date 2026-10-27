//! Log validator (milestone H0.2).
//!
//! Checks the structure of a JSONL file against the schema: the type of
//! each field (done by `serde`), the order of records and basic ranges.
//! It does not judge whether the measurements are good: only whether the file is readable and consistent.

use std::io::BufRead;

use crate::schema::{FrameRecord, Line, Record, RunRecord, SUPPORTED_MAJOR, SummaryRecord};

/// How many problems accumulate before checking stops.
const MAX_ISSUES: usize = 50;

/// A problem found, with its line number (starting at 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

/// Summary of a valid log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// Records read.
    pub records: usize,
    pub frames: u64,
    pub events: u64,
    pub has_summary: bool,
}

/// Validates a complete log.
///
/// # Errors
/// Returns the list of problems (at most 50) if the file does not comply.
pub fn validate<R: BufRead>(reader: R) -> Result<ValidationReport, Vec<ValidationIssue>> {
    let mut state = State::default();

    for (index, raw) in reader.lines().enumerate() {
        let line_no = index + 1;
        match raw {
            Ok(text) => state.check_line(line_no, &text),
            Err(err) => {
                state.issue(line_no, format!("error de lectura: {err}"));
                break;
            }
        }
        if state.issues.len() >= MAX_ISSUES {
            state.issue(line_no, "too many problems; stopping the check".into());
            break;
        }
    }

    if state.records == 0 && state.issues.is_empty() {
        state.issue(1, "the file is empty".into());
    }

    if state.issues.is_empty() {
        Ok(ValidationReport {
            records: state.records,
            frames: state.frames,
            events: state.events,
            has_summary: state.has_summary,
        })
    } else {
        Err(state.issues)
    }
}

#[derive(Default)]
struct State {
    issues: Vec<ValidationIssue>,
    records: usize,
    frames: u64,
    events: u64,
    has_summary: bool,
    last_n: Option<u64>,
    last_present_ms: Option<f64>,
}

impl State {
    fn issue(&mut self, line: usize, message: String) {
        self.issues.push(ValidationIssue { line, message });
    }

    fn check_line(&mut self, line_no: usize, text: &str) {
        if text.trim().is_empty() {
            self.issue(line_no, "empty line".into());
            return;
        }
        let line: Line = match serde_json::from_str(text) {
            Ok(line) => line,
            Err(err) => {
                self.issue(line_no, format!("invalid record: {err}"));
                return;
            }
        };
        self.records += 1;

        if line.schema.split('.').next() != Some(SUPPORTED_MAJOR) {
            self.issue(
                line_no,
                format!(
                    "unsupported schema version: {} (reads {SUPPORTED_MAJOR}.x)",
                    line.schema
                ),
            );
        }
        if self.has_summary {
            self.issue(line_no, "there are records after summary".into());
        }

        let is_run = matches!(line.record, Record::Run(_));
        if self.records == 1 && !is_run {
            self.issue(line_no, "the first record must be of type run".into());
        } else if self.records > 1 && is_run {
            self.issue(line_no, "duplicate run".into());
        }

        match &line.record {
            Record::Run(run) => self.check_run(line_no, run),
            Record::Frame(frame) => self.check_frame(line_no, frame),
            Record::Event(_) => self.events += 1,
            Record::Summary(summary) => {
                self.has_summary = true;
                self.check_summary(line_no, summary);
            }
        }
    }

    fn check_run(&mut self, line_no: usize, run: &RunRecord) {
        if run.run_id.trim().is_empty() {
            self.issue(line_no, "empty run_id".into());
        }
        if !looks_like_iso8601_utc(&run.started_at) {
            self.issue(
                line_no,
                format!("started_at is not ISO 8601 UTC: {}", run.started_at),
            );
        }
        if run.resolution.width() == 0 || run.resolution.height() == 0 {
            self.issue(line_no, "resolution with a zero dimension".into());
        }
    }

    fn check_frame(&mut self, line_no: usize, f: &FrameRecord) {
        self.frames += 1;
        if let Some(prev) = self.last_n.filter(|prev| f.n <= *prev) {
            self.issue(
                line_no,
                format!("n is not increasing: {} after {prev}", f.n),
            );
        }
        self.last_n = Some(f.n);

        if !f.t_render_ms.is_finite() || !f.t_present_ms.is_finite() {
            self.issue(line_no, "non-finite timestamp".into());
        } else if let Some(prev) = self.last_present_ms.filter(|prev| f.t_present_ms < *prev) {
            self.issue(
                line_no,
                format!(
                    "t_present_ms goes backwards: {} after {prev}",
                    f.t_present_ms
                ),
            );
        }
        if f.t_present_ms.is_finite() {
            self.last_present_ms = Some(f.t_present_ms);
        }

        let passes = [
            f.pass_ms.capture,
            f.pass_ms.motion,
            f.pass_ms.reproject,
            f.pass_ms.mask,
            f.pass_ms.present,
        ];
        if passes.iter().any(|ms| !ms.is_finite() || *ms < 0.0) {
            self.issue(line_no, "pass_ms with negative or non-finite values".into());
        }
        if !(0.0..=100.0).contains(&f.repeat_pct) {
            self.issue(
                line_no,
                format!("repeat_pct outside 0 to 100: {}", f.repeat_pct),
            );
        }
        if f.ssim.is_some_and(|v| !(0.0..=1.0).contains(&v)) {
            self.issue(line_no, "ssim outside 0 to 1".into());
        }
        if f.psnr.is_some_and(|v| !v.is_finite() || v < 0.0) {
            self.issue(line_no, "negative or non-finite psnr".into());
        }
    }

    fn check_summary(&mut self, line_no: usize, s: &SummaryRecord) {
        let values = [
            s.fps_rendered_avg,
            s.fps_presented_avg,
            s.p1_ms,
            s.p01_ms,
            s.latency_est_ms.unwrap_or(0.0),
            s.overhead_ms_avg,
        ];
        if values.iter().any(|v| !v.is_finite() || *v < 0.0) {
            self.issue(line_no, "summary with negative or non-finite values".into());
        }
        if let Some(key) = s
            .level_time_pct
            .keys()
            .find(|k| !matches!(k.as_str(), "0" | "1" | "2" | "3"))
        {
            self.issue(line_no, format!("level_time_pct with unknown key: {key}"));
        }
    }
}

/// Shape check `YYYY-MM-DDTHH:MM:SSZ` (does not validate the calendar).
fn looks_like_iso8601_utc(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == 20
        && b[4] == b'-'
        && b[7] == b'-'
        && b[10] == b'T'
        && b[13] == b':'
        && b[16] == b':'
        && b[19] == b'Z'
        && b.iter()
            .enumerate()
            .all(|(i, c)| matches!(i, 4 | 7 | 10 | 13 | 16 | 19) || c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use crate::schema::{EventKind, EventRecord};

    fn to_text(records: Vec<Record>) -> String {
        records
            .into_iter()
            .map(|r| serde_json::to_string(&Line::new(r)).unwrap() + "\n")
            .collect()
    }

    fn valid_log() -> Vec<Record> {
        vec![
            Record::Run(Box::new(fixtures::run())),
            Record::Frame(fixtures::frame(0, 0.0)),
            Record::Event(EventRecord {
                n: 0,
                event: EventKind::SceneCut,
                detail: serde_json::json!({}),
            }),
            Record::Frame(fixtures::frame(1, 16.7)),
            Record::Summary(fixtures::summary()),
        ]
    }

    fn issues(text: &str) -> Vec<String> {
        validate(text.as_bytes())
            .unwrap_err()
            .into_iter()
            .map(|i| i.to_string())
            .collect()
    }

    fn assert_issue(text: &str, needle: &str) {
        let all = issues(text);
        assert!(
            all.iter().any(|m| m.contains(needle)),
            "falta {needle:?} en {all:?}"
        );
    }

    #[test]
    fn accepts_a_valid_log() {
        let report = validate(to_text(valid_log()).as_bytes()).unwrap();
        assert_eq!(
            report,
            ValidationReport {
                records: 5,
                frames: 2,
                events: 1,
                has_summary: true
            }
        );
    }

    #[test]
    fn summary_is_optional() {
        let mut log = valid_log();
        log.pop();
        assert!(!validate(to_text(log).as_bytes()).unwrap().has_summary);
    }

    #[test]
    fn rejects_an_empty_file() {
        assert_issue("", "empty");
    }

    #[test]
    fn rejects_garbage_with_the_line_number() {
        let mut text = to_text(valid_log());
        text.push_str("this is not json\n");
        assert_issue(&text, "line 6: invalid record");
    }

    #[test]
    fn run_must_come_first() {
        let mut log = valid_log();
        log.swap(0, 1);
        assert_issue(&to_text(log), "first record must be of type run");
    }

    #[test]
    fn run_cannot_repeat() {
        let mut log = valid_log();
        log.insert(2, Record::Run(Box::new(fixtures::run())));
        assert_issue(&to_text(log), "duplicate run");
    }

    #[test]
    fn nothing_after_summary() {
        let mut log = valid_log();
        log.push(Record::Frame(fixtures::frame(2, 33.0)));
        assert_issue(&to_text(log), "after summary");
    }

    #[test]
    fn frame_numbers_must_increase() {
        let mut log = valid_log();
        log[3] = Record::Frame(fixtures::frame(0, 16.7));
        assert_issue(&to_text(log), "n is not increasing");
    }

    #[test]
    fn presentation_time_cannot_go_back() {
        let mut log = valid_log();
        log[3] = Record::Frame(fixtures::frame(1, -5.0));
        assert_issue(&to_text(log), "goes backwards");
    }

    #[test]
    fn rejects_an_unsupported_major_version() {
        let text = to_text(valid_log()).replacen("\"schema\":\"1.0\"", "\"schema\":\"2.0\"", 1);
        assert_issue(&text, "unsupported schema version: 2.0");
    }

    #[test]
    fn accepts_a_newer_minor_version() {
        let text = to_text(valid_log()).replace("\"schema\":\"1.0\"", "\"schema\":\"1.7\"");
        assert!(validate(text.as_bytes()).is_ok());
    }

    #[test]
    fn rejects_an_invalid_level() {
        let text = to_text(valid_log()).replacen("\"level\":1", "\"level\":7", 1);
        assert_issue(&text, "invalid level");
    }

    #[test]
    fn rejects_out_of_range_values() {
        let mut bad = fixtures::frame(0, 0.0);
        bad.repeat_pct = 140.0;
        bad.ssim = Some(1.5);
        let mut log = valid_log();
        log[1] = Record::Frame(bad);
        let text = to_text(log);
        assert_issue(&text, "repeat_pct");
        assert_issue(&text, "ssim");
    }

    #[test]
    fn rejects_a_bad_timestamp_and_zero_resolution() {
        let mut run = fixtures::run();
        run.started_at = "ayer".into();
        run.resolution = sv_types::Resolution(0, 1080);
        let mut log = valid_log();
        log[0] = Record::Run(Box::new(run));
        let text = to_text(log);
        assert_issue(&text, "started_at");
        assert_issue(&text, "resolution");
    }

    #[test]
    fn rejects_unknown_summary_level_keys() {
        let mut summary = fixtures::summary();
        summary.level_time_pct.insert("9".into(), 1.0);
        let mut log = valid_log();
        log[4] = Record::Summary(summary);
        assert_issue(&to_text(log), "unknown key");
    }

    #[test]
    fn rejects_blank_lines() {
        let mut text = to_text(valid_log());
        text.push('\n');
        assert_issue(&text, "empty line");
    }

    #[test]
    fn stops_after_many_issues() {
        let text = "basura\n".repeat(500);
        let found = issues(&text);
        assert!(found.len() <= MAX_ISSUES + 1, "{}", found.len());
        assert!(found.last().unwrap().contains("too many"));
    }
}
