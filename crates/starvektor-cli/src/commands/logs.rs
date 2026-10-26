//! `starvektor log`: trabajo con logs de ejecución.

use std::fs::File;
use std::io::BufReader;
use std::process::ExitCode;

use sv_telemetry::validate::validate;

use super::CommandResult;
use crate::cli::LogCommand;

pub fn run(command: LogCommand) -> CommandResult {
    match command {
        LogCommand::Validate { file } => {
            let reader = BufReader::new(
                File::open(&file)
                    .map_err(|err| format!("no se pudo abrir {}: {err}", file.display()))?,
            );
            match validate(reader) {
                Ok(report) => {
                    println!(
                        "ok: {} registros ({} fotogramas, {} eventos, resumen: {})",
                        report.records,
                        report.frames,
                        report.events,
                        if report.has_summary { "sí" } else { "no" }
                    );
                    Ok(ExitCode::SUCCESS)
                }
                Err(issues) => {
                    eprintln!("{} no cumple el esquema:", file.display());
                    for issue in &issues {
                        eprintln!("  {issue}");
                    }
                    Ok(ExitCode::FAILURE)
                }
            }
        }
    }
}
