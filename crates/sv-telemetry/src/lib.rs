//! StarVektor telemetry: versioned JSONL log.
//!
//! One file per run, one line per record. It never contains images or
//! game data (see `docs/data-policy.md`). The schema is described in
//! `docs/log-schema.md`; the source of truth is [`schema`].
//!
//! Modules are independent of each other, except that all use [`schema`]:
//! - [`schema`]: record types (`run`, `frame`, `event`, `summary`).
//! - [`writer`]: asynchronous writing, without adding milliseconds to the frame.
//! - [`summary`]: computes the `summary` record from the `frame` records.
//! - [`validate`]: checks a log against the schema.
//! - [`time`]: ISO 8601 timestamps with no external dependencies.

pub mod schema;
pub mod summary;
pub mod time;
pub mod validate;
pub mod writer;

#[cfg(test)]
mod fixtures;
