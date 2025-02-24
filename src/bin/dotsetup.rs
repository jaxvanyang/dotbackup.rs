use std::process::ExitCode;

use dotbackup::cli::{cli_main, CLIExe};

fn main() -> ExitCode {
	if let Err(e) = cli_main(CLIExe::Dotsetup) {
		eprintln!("{}", e);
		ExitCode::from(1)
	} else {
		ExitCode::SUCCESS
	}
}
