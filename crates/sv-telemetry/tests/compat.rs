//! Schema compatibility.
//!
//! `v1_minimal.jsonl` is a log written according to version 1.0 of the schema. It must
//! keep validating as long as the major version is 1: if this test fails, a
//! change broke compatibility and the major version must be bumped (or
//! the change reverted). See `docs/log-schema.md`.

use sv_telemetry::schema::{Line, Record};
use sv_telemetry::validate::validate;

const V1: &str = include_str!("fixtures/v1_minimal.jsonl");

#[test]
fn a_version_1_log_still_validates() {
    let report = validate(V1.as_bytes()).unwrap_or_else(|issues| {
        panic!("the v1 log no longer validates: {issues:?}");
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
        panic!("the last line must be summary");
    };
    assert_eq!(summary.latency_est_ms, None);
}
