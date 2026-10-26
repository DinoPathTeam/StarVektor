//! Comandos del ejecutable. Cada uno vive en su archivo y no depende de los demás.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sv_config::io::{default_config_path, process_env};

use crate::cli::{Cli, Command};

pub mod config;
pub mod logs;
pub mod selftest;
pub mod tui;

/// Resultado de un comando: código de salida o error a mostrar.
pub type CommandResult = Result<ExitCode, Box<dyn Error>>;

/// Ejecuta el comando pedido.
///
/// # Errors
/// El error del comando, listo para mostrar al usuario.
pub fn dispatch(cli: Cli) -> CommandResult {
    let config = cli.config.as_deref();
    match cli.command {
        Command::Tui => tui::run(config),
        Command::Config(command) => config::run(command, config),
        Command::Log(command) => logs::run(command),
        Command::Selftest(args) => selftest::run(&args, config),
    }
}

/// Ruta del archivo de configuración: la indicada, o la que resuelve el entorno.
pub(crate) fn config_path(explicit: Option<&Path>) -> Result<PathBuf, Box<dyn Error>> {
    match explicit {
        Some(path) => Ok(path.to_path_buf()),
        None => default_config_path(&process_env).ok_or_else(|| {
            "no se pudo determinar la ruta de la configuración: usa --config o define HOME".into()
        }),
    }
}
