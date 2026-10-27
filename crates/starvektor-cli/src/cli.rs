//! Command-line definition. It only describes; it executes nothing.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "starvektor",
    version,
    about = "StarVektor: VRR-style frame pacing"
)]
pub struct Cli {
    /// Configuration file (by default, the one from `config path`).
    #[arg(long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Opens the TUI to edit the configuration.
    Tui,
    /// Queries or creates the configuration file.
    #[command(subcommand)]
    Config(ConfigCommand),
    /// Works with run logs.
    #[command(subcommand)]
    Log(LogCommand),
    /// Runs the system against a synthetic game and validates the log it produces.
    Selftest(SelftestArgs),
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Shows the path of the configuration file.
    Path,
    /// Shows the effective configuration (file plus environment variables).
    Show,
    /// Creates the file with the default values.
    Init {
        /// Overwrites the file if it already exists.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum LogCommand {
    /// Checks that a JSONL log complies with the schema.
    Validate {
        /// `.jsonl` file to validate.
        file: PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct SelftestArgs {
    /// Frames delivered by the synthetic game.
    #[arg(long, default_value_t = 600)]
    pub frames: u64,
    /// Seed of the synthetic game (same seed, same sequence).
    #[arg(long, default_value_t = 1)]
    pub seed: u64,
    /// Where to write the log (by default, the logs directory).
    #[arg(long, value_name = "FILE")]
    pub out: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn the_command_line_definition_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_the_main_forms() {
        let cli = Cli::try_parse_from([
            "starvektor",
            "--config",
            "/x.toml",
            "selftest",
            "--frames",
            "10",
        ])
        .unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/x.toml")));
        assert!(matches!(
            cli.command,
            Command::Selftest(SelftestArgs {
                frames: 10,
                seed: 1,
                out: None
            })
        ));

        let cli = Cli::try_parse_from(["starvektor", "log", "validate", "a.jsonl"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Log(LogCommand::Validate { .. })
        ));

        let cli = Cli::try_parse_from(["starvektor", "config", "init", "--force"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Config(ConfigCommand::Init { force: true })
        ));
    }

    #[test]
    fn rejects_unknown_commands() {
        assert!(Cli::try_parse_from(["starvektor", "fly"]).is_err());
        assert!(Cli::try_parse_from(["starvektor"]).is_err());
    }
}
