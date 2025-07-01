use dotbackup::Cli;
use std::process::ExitCode;

fn main() -> ExitCode {
	let cli = match Cli::dotsetup().parse_args() {
		Ok(cli) => cli,
		Err(e) => {
			eprintln!("{e}");
			return ExitCode::FAILURE;
		}
	};

	if let Err(e) = cli.run() {
		eprintln!("{e}");
		ExitCode::FAILURE
	} else {
		ExitCode::SUCCESS
	}
}
