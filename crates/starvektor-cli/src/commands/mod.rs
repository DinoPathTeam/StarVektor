//! Executable commands. Each lives in its own file and does not depend on the others.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sv_config::io::{default_config_path, process_env};

use crate::cli::{Cli, Command};

pub mod config;
pub mod logs;
pub mod measure;
pub mod selftest;
pub mod tui;

/// Result of a command: exit code or error to display.
pub type CommandResult = Result<ExitCode, Box<dyn Error>>;

/// Runs the requested command.
///
/// # Errors
/// The command's error, ready to show to the user.
pub fn dispatch(cli: Cli) -> CommandResult {
    let config = cli.config.as_deref();
    match cli.command {
        Command::Tui => tui::run(config),
        Command::Config(command) => config::run(command, config),
        Command::Log(command) => logs::run(command),
        Command::Selftest(args) => selftest::run(&args, config),
        Command::Measure(args) => measure::run(&args),
    }
}

/// Path of the configuration file: the one given, or the one the environment resolves.
pub(crate) fn config_path(explicit: Option<&Path>) -> Result<PathBuf, Box<dyn Error>> {
    match explicit {
        Some(path) => Ok(path.to_path_buf()),
        None => default_config_path(&process_env).ok_or_else(|| {
            "could not determine the configuration path: use --config or set HOME".into()
        }),
    }
}
