use std::env;
use std::fmt::Display;

use crate::config::Config;
use crate::{AppError, CLIOpt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CLIExe {
	Dotbackup,
	Dotsetup,
}

impl Display for CLIExe {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Dotbackup => write!(f, "dotbackup"),
			Self::Dotsetup => write!(f, "dotsetup"),
		}
	}
}

pub fn cli_main(exe: CLIExe) -> Result<(), AppError> {
	let mut opt = match exe {
		CLIExe::Dotbackup => CLIOpt::Backup,
		CLIExe::Dotsetup => CLIOpt::Setup,
	};

	let mut args = env::args();
	eprintln!(
		"LOG: executable path: {}",
		args.next().expect("there should be at least 1 argument")
	);
	let config = Config::parse_args(&mut args, &mut opt)?;

	match opt {
		CLIOpt::Backup => {
			config.unwrap().backup()?;
		}
		CLIOpt::Setup => {
			config.unwrap().setup()?;
		}
		CLIOpt::Help => {
			Config::help(&exe);
		}
		CLIOpt::List => {
			config.unwrap().list_apps();
		}
		CLIOpt::Version => {
			Config::version();
		}
		CLIOpt::DumpConfig => {
			print!("{}", config.unwrap());
		}
	}

	Ok(())
}
