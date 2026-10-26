//! `starvektor tui`: abre la TUI de configuración.

use std::io::{IsTerminal, stdin, stdout};
use std::path::Path;
use std::process::ExitCode;

use sv_config::Config;

use super::{CommandResult, config_path};

pub fn run(explicit: Option<&Path>) -> CommandResult {
    if !stdin().is_terminal() || !stdout().is_terminal() {
        return Err("la TUI necesita un terminal interactivo; para editar sin ella usa `starvektor config path` y el archivo TOML".into());
    }
    let path = config_path(explicit)?;
    // Se edita el archivo tal cual: las variables de entorno no se mezclan aquí.
    let config = Config::load_or_default(&path)?;
    sv_tui::run(config, path)?;
    Ok(ExitCode::SUCCESS)
}
