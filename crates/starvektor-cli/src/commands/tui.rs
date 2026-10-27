//! `starvektor tui`: opens the configuration TUI.

use std::io::{IsTerminal, stdin, stdout};
use std::path::Path;
use std::process::ExitCode;

use sv_config::Config;

use super::{CommandResult, config_path};

pub fn run(explicit: Option<&Path>) -> CommandResult {
    if !stdin().is_terminal() || !stdout().is_terminal() {
        return Err("the TUI needs an interactive terminal; to edit without it, use `starvektor config path` and the TOML file".into());
    }
    let path = config_path(explicit)?;
    // The file is edited as is: environment variables are not mixed in here.
    let config = Config::load_or_default(&path)?;
    sv_tui::run(config, path)?;
    Ok(ExitCode::SUCCESS)
}
