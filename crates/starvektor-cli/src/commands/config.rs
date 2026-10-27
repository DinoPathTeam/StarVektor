//! `starvektor config`: path, effective content and file creation.

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
                    "{} already exists; use --force to overwrite it",
                    path.display()
                )
                .into());
            }
            Config::default().save(&path)?;
            println!("configuration created at {}", path.display());
        }
    }
    Ok(ExitCode::SUCCESS)
}
