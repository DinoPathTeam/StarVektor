//! Measurement sources on Linux, read **from outside the game process**.
//!
//! - [`mangoapp`]: parser for the per-frame CSV that `mangoapp` writes next to
//!   gamescope (a separate process, compositor side).
//! - [`sysfs`]: GPU memory counters exposed by the `amdgpu` kernel driver.
//!
//! Nothing here touches game memory, files or network: it only reads files that
//! other tools produce about their own state. Phase 1 (H1.1) uses it to measure
//! without generating anything.

pub mod mangoapp;
pub mod sysfs;
