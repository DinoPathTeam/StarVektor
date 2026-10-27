//! `starvektor log`: working with run logs.

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
                    .map_err(|err| format!("could not open {}: {err}", file.display()))?,
            );
            match validate(reader) {
                Ok(report) => {
                    println!(
                        "ok: {} records ({} frames, {} events, summary: {})",
                        report.records,
                        report.frames,
                        report.events,
                        if report.has_summary { "yes" } else { "no" }
                    );
                    Ok(ExitCode::SUCCESS)
                }
                Err(issues) => {
                    eprintln!("{} does not comply with the schema:", file.display());
                    for issue in &issues {
                        eprintln!("  {issue}");
                    }
                    Ok(ExitCode::FAILURE)
                }
            }
        }
    }
}
