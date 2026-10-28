//! StarVektor composition root.
//!
//! It is the only crate that knows all the others. It has no logic of its own:
//! it converts the types of one layer to those of another and connects the pieces.
//! Each command lives in its own file under [`commands`], so adding or changing one
//! does not touch the others.
//!
//! - [`wiring`]: translation between layers (configuration -> core parameters).
//! - [`ids`]: run identifiers.
//! - [`commands`]: `tui`, `config`, `log`, `selftest` and `measure`.

pub mod cli;
pub mod commands;
pub mod ids;
pub mod wiring;
