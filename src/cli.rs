pub mod action;
pub mod config;

pub use action::*;
pub use config::*;

use crate::{VERSION, arg_error, error::Result, sys_error};
use std::{
	env,
	path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Name {
	#[default]
	Dotbackup,
	Dotsetup,
}

impl std::fmt::Display for Name {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Name::Dotbackup => write!(f, "dotbackup"),
			Name::Dotsetup => write!(f, "dotsetup"),
		}
	}
}

/// The main command-line interface (dotbackup & dotsetup).
#[derive(Debug, Clone, Default)]
pub struct Cli {
	/// dotbackup or dotsetup
	pub name: Name,
	pub config: Config,
	pub action: Action,
}

impl Cli {
	#[must_use]
	pub fn new(name: Name) -> Self {
		let action = match name {
			Name::Dotbackup => Action::Backup,
			Name::Dotsetup => Action::Setup,
		};

		Self {
			name,
			action,
			..Default::default()
		}
	}

	/// Create a new dotbackup CLI (alias of `default()`).
	#[must_use]
	pub fn dotbackup() -> Self {
		Self::default()
	}

	/// Create a new dotsetup CLI.
	#[must_use]
	pub fn dotsetup() -> Self {
		Self::new(Name::Dotsetup)
	}

	#[must_use]
	pub fn action(mut self, action: Action) -> Self {
		self.action = action;
		self
	}

	pub fn apply_config(&mut self, config_path: &Path) -> Result<()> {
		let config = Config::from_file(config_path)?;
		let clean = if self.config.clean {
			true
		} else {
			config.clean
		};

		self.config = Config {
			clean,
			selected_apps: self.config.selected_apps.clone(),
			..config
		};

		Ok(())
	}

	pub fn config_dir() -> Result<PathBuf> {
		Ok(dirs::config_dir()
			.ok_or(sys_error!(
				"unknown system, cannot decide configuration directory"
			))?
			.join("dotbackup"))
	}

	pub fn default_config_path() -> Result<PathBuf> {
		Ok(Self::config_dir()?.join("dotbackup.yml"))
	}

	/// Parse command-line arguments and also parse config.
	pub fn parse_args(mut self) -> Result<Self> {
		let mut config_parsed = false;
		let mut args = env::args();
		args.next();

		while let Some(arg) = args.next() {
			match &arg[..] {
				"-h" | "--help" => {
					return Ok(self.action(Action::Help));
				}
				"-f" | "--file" => {
					let file_path = args
						.next()
						.ok_or(arg_error!("expected a file path after option {arg}"))?;
					self.apply_config(&PathBuf::from(file_path))?;
					config_parsed = true;
				}
				"-c" | "--config" => {
					let config_name = args.next().ok_or(arg_error!(
						"expected a configuration name after option {arg}"
					))?;
					self.apply_config(&Self::config_dir()?.join(format!("{config_name}.yml")))?;
					config_parsed = true;
				}
				"-l" | "--list" => {
					self.action = Action::List;
				}
				"--clean" => self.config.clean = true,
				"-V" | "--version" => return Ok(self.action(Action::Version)),
				"-v" | "--verbose" => todo!(),
				"--dump-config" => self.action = Action::DumpConfig,
				_ => {
					if arg.starts_with('-') {
						return Err(arg_error!("unknown argument: {arg:?}"));
					}
					self.config.selected_apps.push(arg);
				}
			}
		}

		if !config_parsed {
			self.apply_config(&Self::default_config_path()?)?;
		}

		Ok(self)
	}

	#[allow(clippy::unit_arg)]
	pub fn run(&self) -> Result<()> {
		match self.action {
			Action::Backup => self.config.backup(),
			Action::Setup => self.config.setup(),
			Action::Help => self.help(),
			Action::List => Ok(self.config.list_apps()),
			Action::Version => Ok(println!("{} {VERSION}", self.name)),
			Action::DumpConfig => Ok(print!("{}", self.config)),
		}
	}

	/// Print help message.
	pub fn help(&self) -> Result<()> {
		let clean_help = match self.name {
			Name::Dotbackup => "Delete old backup files before backup",
			Name::Dotsetup => "Delete old configuration files before setup",
		};
		let config_path = Self::config_dir()?.join("CONFIG.yml");

		print!(
			"\
Dotfile backup utility (dotbackup / dotsetup)

Usage: {name} [OPTIONS] [APPS]

Options:
  -h, --help                     Print help
  -f, --file <PATH>              Use configuration file at PATH
  -c, --config <CONFIG>          Use configuration file at {config_path}
  -l, --list                     List all applications and exit
      --clean                    {clean_help}
  -V, --version                  Print version info and exit
  -v, --verbose                  Use verbose output
      --dump-config              Print parsed configuration

See 'man dotbackup' and 'man dotsetup' for more information.
",
			name = self.name,
			config_path = config_path.display()
		);

		Ok(())
	}
}
