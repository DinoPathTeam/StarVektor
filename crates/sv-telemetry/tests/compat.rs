//! Compatibilidad del esquema.
//!
//! `v1_minimal.jsonl` es un log escrito según la versión 1.0 del esquema. Debe
//! seguir validando mientras la versión mayor sea 1: si esta prueba falla, un
//! cambio rompió la compatibilidad y hay que subir la versión mayor (o
//! revertir). Ver `docs/log-schema.md`.

use sv_telemetry::schema::{Line, Record};
use sv_telemetry::validate::validate;

const V1: &str = include_str!("fixtures/v1_minimal.jsonl");

#[test]
fn a_version_1_log_still_validates() {
    let report = validate(V1.as_bytes()).unwrap_or_else(|issues| {
        panic!("el log v1 dejó de validar: {issues:?}");
    });
    assert_eq!(report.records, 6);
    assert_eq!(report.frames, 3);
    assert_eq!(report.events, 1);
    assert!(report.has_summary);
}

#[test]
fn every_line_parses_into_the_expected_record() {
    let kinds: Vec<&str> = V1
        .lines()
        .map(
            |line| match serde_json::from_str::<Line>(line).unwrap().record {
                Record::Run(_) => "run",
                Record::Frame(_) => "frame",
                Record::Event(_) => "event",
                Record::Summary(_) => "summary",
            },
        )
        .collect();
    assert_eq!(
        kinds,
        ["run", "frame", "frame", "frame", "event", "summary"]
    );
}

#[test]
fn optional_fields_may_be_absent() {
    let summary = V1.lines().last().unwrap();
    let Line {
        record: Record::Summary(summary),
        ..
    } = serde_json::from_str(summary).unwrap()
    else {
        panic!("la última línea debe ser summary");
    };
    assert_eq!(summary.latency_est_ms, None);
}
