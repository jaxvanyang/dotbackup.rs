mod app;
use std::fs::File;
use std::io::Read;
use std::{fmt::Display, path::PathBuf};

pub use app::*;
use dirs::{config_dir, home_dir};
use expanduser::expanduser;
use glob::Pattern;
use saphyr::Yaml;

use crate::cli::CLIExe;
use crate::{run_hook, AppError, CLIOpt, VERSION};

#[derive(Debug, Default)]
pub struct Config {
	pub backup_dir: Option<PathBuf>,
	pub clean: bool,
	pub ignore: Vec<Pattern>,
	pub selected_apps: Vec<String>,
	pub apps: Vec<App>,
	pub pre_backup: Vec<String>,
	pub post_backup: Vec<String>,
	pub pre_setup: Vec<String>,
	pub post_setup: Vec<String>,
}

impl Config {
	fn collect_array(hash: &saphyr::Hash, key: &str) -> Result<Vec<String>, AppError> {
		match hash.get(&Yaml::from_str(key)) {
			Some(Yaml::Array(array)) => {
				let mut vec = Vec::new();
				for s in array {
					if let Yaml::String(s) = s {
						vec.push(s.clone());
					} else {
						return Err(AppError::ConfigError(format!(
							"element of {key} must be of type String, but found: {s:?}",
						)));
					}
				}
				Ok(vec)
			}
			None => Ok(Vec::new()),
			_ => Err(AppError::ConfigError(format!("{} should be an array", key))),
		}
	}

	fn format_array(name: &str, array: &Vec<String>, indentation: usize) -> String {
		let mut output = String::new();
		if array.is_empty() {
			return output;
		}

		let mut indentation = "  ".repeat(indentation);
		output.push_str(format!("{indentation}{name}:\n").as_str());

		indentation.push_str("  ");
		for s in array {
			output.push_str(format!("{indentation}- {s}\n").as_str());
		}

		output
	}

	pub fn config_dir() -> PathBuf {
		config_dir()
			.expect("this platform should specify a configuration directory")
			.join("dotbackup")
	}

	pub fn default_path() -> PathBuf {
		Self::config_dir().join("dotbackup.yml")
	}

	pub fn list_apps(&self) {
		for app in &self.apps {
			println!("{}", app.name);
		}
	}

	pub fn backup(&self) -> Result<(), AppError> {
		let backup_dir = self
			.backup_dir
			.clone()
			.ok_or(AppError::config_error("backup_dir not set"))?;
		let config_root = home_dir().ok_or(AppError::sys_error("home directory unknown"))?;

		let n = self.pre_backup.len();
		for (i, hook) in self.pre_backup.iter().enumerate() {
			println!(":: Running pre-backup hooks ({}/{n})...", i + 1);
			run_hook(hook, &backup_dir)?;
		}

		if self.selected_apps.is_empty() {
			for app in &self.apps {
				app.backup(&config_root, &backup_dir, self.clean, &self.ignore)?;
			}
		} else {
			for selected in &self.selected_apps {
				for app in &self.apps {
					if *selected == app.name {
						app.backup(&config_root, &backup_dir, self.clean, &self.ignore)?;
					}
				}
			}
		}

		let n = self.post_backup.len();
		for (i, hook) in self.post_backup.iter().enumerate() {
			println!(":: Running post-backup hooks ({}/{n})...", i + 1);
			run_hook(hook, &backup_dir)?;
		}

		Ok(())
	}

	pub fn setup(&self) -> Result<(), AppError> {
		let backup_dir = self
			.backup_dir
			.clone()
			.ok_or(AppError::config_error("backup_dir not set"))?;
		let config_root = home_dir().ok_or(AppError::sys_error("home directory unknown"))?;

		let n = self.pre_setup.len();
		for (i, hook) in self.pre_setup.iter().enumerate() {
			println!(":: Running pre-setup hooks ({}/{n})...", i + 1);
			run_hook(hook, &backup_dir)?;
		}

		if self.selected_apps.is_empty() {
			for app in &self.apps {
				app.setup(&config_root, &backup_dir, self.clean, &self.ignore)?;
			}
		} else {
			for selected in &self.selected_apps {
				for app in &self.apps {
					if *selected == app.name {
						app.setup(&config_root, &backup_dir, self.clean, &self.ignore)?;
					}
				}
			}
		}

		let n = self.post_setup.len();
		for (i, hook) in self.post_setup.iter().enumerate() {
			println!(":: Running post-setup hooks ({}/{n})...", i + 1);
			run_hook(hook, &backup_dir)?;
		}

		Ok(())
	}

	pub fn parse_args<I: Iterator<Item = String>>(
		args: &mut I,
		opt: &mut CLIOpt,
	) -> Result<Option<Self>, AppError> {
		let mut config_path = Self::default_path();
		let mut selected_apps = Vec::new();
		let mut clean = false;

		while let Some(arg) = args.next() {
			match &arg[..] {
				"-h" | "--help" => {
					*opt = CLIOpt::Help;
					return Ok(None);
				}
				"-f" | "--file" => {
					let file_path = args.next().ok_or(AppError::ArgError(format!(
						"option {arg} should be followed by a path"
					)))?;
					config_path = PathBuf::from(file_path);
				}
				"-c" | "--config" => {
					let config_name = args.next().ok_or(AppError::ArgError(format!(
						"option {arg} should be followed by a configuration name"
					)))?;
					config_path =
						Self::config_dir().join(PathBuf::from(format!("{config_name}.yml")));
				}
				"-l" | "--list" => {
					*opt = CLIOpt::List;
				}
				"--clean" => clean = true,
				"-V" | "--version" => {
					*opt = CLIOpt::Version;
					return Ok(None);
				}
				"-v" | "--verbose" => todo!(),
				"--dump-config" => {
					*opt = CLIOpt::DumpConfig;
				}
				_ => {
					if arg.starts_with("-") {
						return Err(AppError::ArgError(format!("unknown argument: {arg:?}")));
					} else {
						selected_apps.push(arg);
					}
				}
			}
		}

		let mut config_file = File::open(&config_path)
			.map_err(|e| AppError::SysError(format!("{e}: {}", config_path.to_str().unwrap())))?;
		let mut config = String::new();
		config_file
			.read_to_string(&mut config)
			.map_err(|e| AppError::SysError(e.to_string()))?;
		let mut config = Config::try_from(&config)?;

		if clean {
			config.clean = true;
		}

		for selected in &selected_apps {
			let mut found = false;
			for app in &config.apps {
				if *selected == app.name {
					found = true;
					break;
				}
			}
			if !found {
				return Err(AppError::ArgError(format!(
					"selected app not configured: {selected}"
				)));
			}
		}
		config.selected_apps = selected_apps;

		Ok(Some(config))
	}

	pub fn version() {
		println!("dotbackup {VERSION}");
	}

	pub fn help(exe: CLIExe) {
		let clean_help = match exe {
			CLIExe::Dotbackup => "Delete old backup files before backup",
			CLIExe::Dotsetup => "Delete old configuration files before setup",
		};
		let config_path = config_dir()
			.expect("this platform should have a configuration directory")
			.join("dotbackup")
			.join("CONFIG.yml")
			.to_str()
			.unwrap()
			.to_string();
		print!(
			"\
Dotfile backup utility (dotbackup & dotsetup)

Usage: {exe} [OPTIONS] [APPS]

Options:
  -h, --help                     Print help
  -f, --file <PATH>              Use configuration file at PATH
  -c, --config <CONFIG>          Use configuration file at {config_path}
  -l, --list                     List applications and exit
      --clean                    {clean_help}
  -V, --version                  Print version info and exit
  -v, --verbose                  Use verbose output
      --dump-config              Print parsed configuration, useful for debugging

See 'man dotbackup' and 'man dotsetup' for more information.
"
		);
	}
}

impl Display for Config {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(backup_dir) = &self.backup_dir {
			writeln!(
				f,
				"backup_dir: {}",
				backup_dir
					.to_str()
					.expect("backup_dir should be UTF-8 encoded")
			)?;
		}

		writeln!(f, "clean: {}", self.clean)?;

		write!(
			f,
			"{}",
			Self::format_array(
				"ignore",
				&self.ignore.iter().map(|p| p.to_string()).collect(),
				0
			)
		)?;

		if !self.selected_apps.is_empty() {
			writeln!(f, "# selected_apps: {:?}", self.selected_apps)?;
		}

		if !self.apps.is_empty() {
			writeln!(f, "apps:")?;
			for app in &self.apps {
				write!(f, "{}", app)?;
			}
		}

		write!(
			f,
			"{}",
			Config::format_array("pre_backup", &self.pre_backup, 0)
		)?;
		write!(
			f,
			"{}",
			Config::format_array("post_backup", &self.post_backup, 0)
		)?;
		write!(
			f,
			"{}",
			Config::format_array("pre_setup", &self.pre_setup, 0)
		)?;
		write!(
			f,
			"{}",
			Config::format_array("post_setup", &self.post_setup, 0)
		)
	}
}

impl TryFrom<&String> for Config {
	type Error = AppError;
	fn try_from(value: &String) -> Result<Self, Self::Error> {
		let yaml =
			&Yaml::load_from_str(value).map_err(|e| Self::Error::ConfigError(e.to_string()))?[0];

		if let Yaml::Hash(yaml) = yaml {
			let mut backup_dir = None;
			let mut clean = false;
			let mut ignore = Vec::new();

			match yaml.get(&Yaml::from_str("backup_dir")) {
				Some(Yaml::String(s)) => {
					backup_dir =
						Some(expanduser(s).map_err(|e| {
							Self::Error::ConfigError(format!("expanduser error: {e}"))
						})?);
				}
				None => (),
				_ => {
					return Err(Self::Error::ConfigError(
						"backup_dir should be a string".to_string(),
					))
				}
			}

			match yaml.get(&Yaml::from_str("clean")) {
				Some(Yaml::Boolean(b)) => clean = *b,
				None => (),
				_ => {
					return Err(Self::Error::ConfigError(
						"clean should be a boolean".to_string(),
					))
				}
			}

			for glob in Config::collect_array(yaml, "ignore")? {
				ignore.push(
					Pattern::new(&glob)
						.map_err(|e| AppError::ConfigError(format!("invalid glob: {e}")))?,
				);
			}

			let mut apps = Vec::new();
			match yaml.get(&Yaml::from_str("apps")) {
				Some(Yaml::Hash(app_hash)) => {
					for (name, value) in app_hash {
						if let Yaml::String(name) = name {
							let app = App::load_from_config(name, value)?;
							apps.push(app);
						} else {
							return Err(Self::Error::ConfigError(
								"app name should be a string".to_string(),
							));
						}
					}
				}
				None => (),
				_ => {
					return Err(Self::Error::ConfigError(
						"apps should be a mapping".to_string(),
					))
				}
			}

			Ok(Self {
				backup_dir,
				clean,
				ignore,
				selected_apps: Vec::new(),
				apps,
				pre_backup: Config::collect_array(yaml, "pre_backup")?,
				post_backup: Config::collect_array(yaml, "post_backup")?,
				pre_setup: Config::collect_array(yaml, "pre_setup")?,
				post_setup: Config::collect_array(yaml, "post_setup")?,
			})
		} else {
			Err(Self::Error::ConfigError(
				"configuration not valid".to_string(),
			))
		}
	}
}
