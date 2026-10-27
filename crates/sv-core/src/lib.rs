//! Agnostic core of StarVektor.
//!
//! There are no operating system or graphics API calls here: everything that
//! comes in is numbers and everything that goes out is decisions. That makes it
//! testable without hardware and portable to any system (see
//! `docs/architecture.md`).
//!
//! Each module is independent of the others:
//! - [`timing`]: frame-time prediction.
//! - [`regulator`]: raises and lowers the level based on cost and memory (with hysteresis).
//! - [`chain`]: detects whether the chain includes upscaling or frame generation.
//!
//! The confidence mask and motion estimation (Phase 2) will get their own
//! module; none of the above will be touched to add them.

pub mod chain;
pub mod regulator;
pub mod timing;
