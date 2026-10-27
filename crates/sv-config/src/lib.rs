//! StarVektor configuration (decision D3).
//!
//! Everything configurable lives in a TOML file and in environment variables; the
//! TUI (`sv-tui`) only edits it. This way it works the same from a Steam launch
//! option or a script, without opening anything.
//!
//! This crate does not know `sv-core`: the model is plain data and the conversion to
//! the core types is done in the composition crate. That is why the regulator can
//! change without touching the configuration, and vice versa.
//!
//! - [`model`]: structures and default values.
//! - [`effective`]: per-game profile resolution.
//! - [`validate`]: range rules.
//! - [`env`]: overrides from environment variables.
//! - [`io`]: loading, atomic saving and default paths.

pub mod effective;
pub mod env;
pub mod error;
pub mod io;
pub mod model;
pub mod validate;

pub use effective::Effective;
pub use error::{ConfigError, ConfigIssue};
pub use model::{Config, HookMode};
