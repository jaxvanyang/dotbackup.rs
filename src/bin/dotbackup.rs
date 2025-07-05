use dotbackup::{Cli, error};
use std::process::ExitCode;

fn main() -> ExitCode {
	let cli = match Cli::default().parse_args() {
		Ok(cli) => cli,
		Err(e) => {
			error!("{e}");
			return ExitCode::FAILURE;
		}
	};

	if let Err(e) = cli.run() {
		error!("{e}");
		ExitCode::FAILURE
	} else {
		ExitCode::SUCCESS
	}
}
