//! Definición de la línea de comandos. Solo describe; no ejecuta nada.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "starvektor",
    version,
    about = "StarVektor: sincronización de fotogramas al estilo VRR"
)]
pub struct Cli {
    /// Archivo de configuración (por defecto, el de `config path`).
    #[arg(long, global = true, value_name = "ARCHIVO")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Abre la TUI para editar la configuración.
    Tui,
    /// Consulta o crea el archivo de configuración.
    #[command(subcommand)]
    Config(ConfigCommand),
    /// Trabaja con logs de ejecución.
    #[command(subcommand)]
    Log(LogCommand),
    /// Ejecuta el sistema contra un juego sintético y valida el log que produce.
    Selftest(SelftestArgs),
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Muestra la ruta del archivo de configuración.
    Path,
    /// Muestra la configuración vigente (archivo más variables de entorno).
    Show,
    /// Crea el archivo con los valores por defecto.
    Init {
        /// Sobrescribe el archivo si ya existe.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum LogCommand {
    /// Comprueba que un log JSONL cumple el esquema.
    Validate {
        /// Archivo `.jsonl` a validar.
        file: PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct SelftestArgs {
    /// Fotogramas que entrega el juego sintético.
    #[arg(long, default_value_t = 600)]
    pub frames: u64,
    /// Semilla del juego sintético (misma semilla, misma secuencia).
    #[arg(long, default_value_t = 1)]
    pub seed: u64,
    /// Dónde escribir el log (por defecto, el directorio de logs).
    #[arg(long, value_name = "ARCHIVO")]
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
        assert!(Cli::try_parse_from(["starvektor", "volar"]).is_err());
        assert!(Cli::try_parse_from(["starvektor"]).is_err());
    }
}
