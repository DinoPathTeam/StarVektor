//! Validador de logs (hito H0.2).
//!
//! Comprueba la estructura de un archivo JSONL contra el esquema: el tipo de
//! cada campo (lo hace `serde`), el orden de los registros y rangos básicos.
//! No juzga si las medidas son buenas: solo si el archivo es legible y coherente.

use std::io::BufRead;

use crate::schema::{FrameRecord, Line, Record, RunRecord, SUPPORTED_MAJOR, SummaryRecord};

/// Cuántos problemas se acumulan antes de dejar de revisar.
const MAX_ISSUES: usize = 50;

/// Un problema encontrado, con su número de línea (empieza en 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "línea {}: {}", self.line, self.message)
    }
}

/// Resumen de un log válido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// Registros leídos.
    pub records: usize,
    pub frames: u64,
    pub events: u64,
    pub has_summary: bool,
}

/// Valida un log completo.
///
/// # Errors
/// Devuelve la lista de problemas (como máximo 50) si el archivo no cumple.
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
            state.issue(line_no, "demasiados problemas; se deja de revisar".into());
            break;
        }
    }

    if state.records == 0 && state.issues.is_empty() {
        state.issue(1, "el archivo está vacío".into());
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
            self.issue(line_no, "línea vacía".into());
            return;
        }
        let line: Line = match serde_json::from_str(text) {
            Ok(line) => line,
            Err(err) => {
                self.issue(line_no, format!("registro inválido: {err}"));
                return;
            }
        };
        self.records += 1;

        if line.schema.split('.').next() != Some(SUPPORTED_MAJOR) {
            self.issue(
                line_no,
                format!(
                    "versión de esquema no soportada: {} (se lee {SUPPORTED_MAJOR}.x)",
                    line.schema
                ),
            );
        }
        if self.has_summary {
            self.issue(line_no, "hay registros después de summary".into());
        }

        let is_run = matches!(line.record, Record::Run(_));
        if self.records == 1 && !is_run {
            self.issue(line_no, "el primer registro debe ser de tipo run".into());
        } else if self.records > 1 && is_run {
            self.issue(line_no, "run repetido".into());
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
            self.issue(line_no, "run_id vacío".into());
        }
        if !looks_like_iso8601_utc(&run.started_at) {
            self.issue(
                line_no,
                format!("started_at no es ISO 8601 UTC: {}", run.started_at),
            );
        }
        if run.resolution.width() == 0 || run.resolution.height() == 0 {
            self.issue(line_no, "resolution con dimensión cero".into());
        }
    }

    fn check_frame(&mut self, line_no: usize, f: &FrameRecord) {
        self.frames += 1;
        if let Some(prev) = self.last_n.filter(|prev| f.n <= *prev) {
            self.issue(
                line_no,
                format!("n no es creciente: {} después de {prev}", f.n),
            );
        }
        self.last_n = Some(f.n);

        if !f.t_render_ms.is_finite() || !f.t_present_ms.is_finite() {
            self.issue(line_no, "marca de tiempo no finita".into());
        } else if let Some(prev) = self.last_present_ms.filter(|prev| f.t_present_ms < *prev) {
            self.issue(
                line_no,
                format!(
                    "t_present_ms retrocede: {} después de {prev}",
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
            self.issue(line_no, "pass_ms con valores negativos o no finitos".into());
        }
        if !(0.0..=100.0).contains(&f.repeat_pct) {
            self.issue(
                line_no,
                format!("repeat_pct fuera de 0 a 100: {}", f.repeat_pct),
            );
        }
        if f.ssim.is_some_and(|v| !(0.0..=1.0).contains(&v)) {
            self.issue(line_no, "ssim fuera de 0 a 1".into());
        }
        if f.psnr.is_some_and(|v| !v.is_finite() || v < 0.0) {
            self.issue(line_no, "psnr negativo o no finito".into());
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
            self.issue(line_no, "summary con valores negativos o no finitos".into());
        }
        if let Some(key) = s
            .level_time_pct
            .keys()
            .find(|k| !matches!(k.as_str(), "0" | "1" | "2" | "3"))
        {
            self.issue(
                line_no,
                format!("level_time_pct con clave desconocida: {key}"),
            );
        }
    }
}

/// Comprobación de forma `YYYY-MM-DDTHH:MM:SSZ` (no valida el calendario).
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
        assert_issue("", "vacío");
    }

    #[test]
    fn rejects_garbage_with_the_line_number() {
        let mut text = to_text(valid_log());
        text.push_str("esto no es json\n");
        assert_issue(&text, "línea 6: registro inválido");
    }

    #[test]
    fn run_must_come_first() {
        let mut log = valid_log();
        log.swap(0, 1);
        assert_issue(&to_text(log), "primer registro debe ser de tipo run");
    }

    #[test]
    fn run_cannot_repeat() {
        let mut log = valid_log();
        log.insert(2, Record::Run(Box::new(fixtures::run())));
        assert_issue(&to_text(log), "run repetido");
    }

    #[test]
    fn nothing_after_summary() {
        let mut log = valid_log();
        log.push(Record::Frame(fixtures::frame(2, 33.0)));
        assert_issue(&to_text(log), "después de summary");
    }

    #[test]
    fn frame_numbers_must_increase() {
        let mut log = valid_log();
        log[3] = Record::Frame(fixtures::frame(0, 16.7));
        assert_issue(&to_text(log), "n no es creciente");
    }

    #[test]
    fn presentation_time_cannot_go_back() {
        let mut log = valid_log();
        log[3] = Record::Frame(fixtures::frame(1, -5.0));
        assert_issue(&to_text(log), "retrocede");
    }

    #[test]
    fn rejects_an_unsupported_major_version() {
        let text = to_text(valid_log()).replacen("\"schema\":\"1.0\"", "\"schema\":\"2.0\"", 1);
        assert_issue(&text, "versión de esquema no soportada: 2.0");
    }

    #[test]
    fn accepts_a_newer_minor_version() {
        let text = to_text(valid_log()).replace("\"schema\":\"1.0\"", "\"schema\":\"1.7\"");
        assert!(validate(text.as_bytes()).is_ok());
    }

    #[test]
    fn rejects_an_invalid_level() {
        let text = to_text(valid_log()).replacen("\"level\":1", "\"level\":7", 1);
        assert_issue(&text, "nivel inválido");
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
        assert_issue(&to_text(log), "clave desconocida");
    }

    #[test]
    fn rejects_blank_lines() {
        let mut text = to_text(valid_log());
        text.push('\n');
        assert_issue(&text, "línea vacía");
    }

    #[test]
    fn stops_after_many_issues() {
        let text = "basura\n".repeat(500);
        let found = issues(&text);
        assert!(found.len() <= MAX_ISSUES + 1, "{}", found.len());
        assert!(found.last().unwrap().contains("demasiados"));
    }
}
