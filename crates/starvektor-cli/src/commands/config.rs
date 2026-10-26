//! `starvektor config`: ruta, contenido vigente y creación del archivo.

use std::path::Path;
use std::process::ExitCode;

use sv_config::Config;
use sv_config::io::process_env;

use super::{CommandResult, config_path};
use crate::cli::ConfigCommand;

pub fn run(command: ConfigCommand, explicit: Option<&Path>) -> CommandResult {
    let path = config_path(explicit)?;
    match command {
        ConfigCommand::Path => println!("{}", path.display()),
        ConfigCommand::Show => {
            let mut config = Config::load_or_default(&path)?;
            config.apply_env(&process_env)?;
            print!("{}", config.to_toml_string()?);
        }
        ConfigCommand::Init { force } => {
            if path.exists() && !force {
                return Err(format!(
                    "{} ya existe; usa --force para sobrescribirlo",
                    path.display()
                )
                .into());
            }
            Config::default().save(&path)?;
            println!("configuración creada en {}", path.display());
        }
    }
    Ok(ExitCode::SUCCESS)
}
