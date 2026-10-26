use std::process::ExitCode;

use clap::Parser;
use starvektor_cli::cli::Cli;
use starvektor_cli::commands::dispatch;

fn main() -> ExitCode {
    match dispatch(Cli::parse()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
