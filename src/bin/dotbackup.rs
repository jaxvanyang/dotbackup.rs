use std::process::ExitCode;

use dotbackup::cli::{CLIExe, cli_main};

fn main() -> ExitCode {
	if let Err(e) = cli_main(CLIExe::Dotbackup) {
		eprintln!("{e}");
		ExitCode::from(1)
	} else {
		ExitCode::SUCCESS
	}
}
