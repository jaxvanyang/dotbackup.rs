mod app;

pub use app::*;

use crate::{
	arg_error, config_error,
	error::{Error, Result},
	run_hook, sys_error,
};
use expanduser::expanduser;
use glob::Pattern;
use saphyr::Yaml;
use std::{
	fs::File,
	io::Read,
	path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default)]
pub struct Config {
	// CLI args may change these
	pub selected_apps: Vec<String>,
	pub clean: bool,

	pub backup_dir: Option<PathBuf>,
	pub ignore: Vec<Pattern>,
	pub apps: Vec<App>,
	pub pre_backup: Vec<String>,
	pub post_backup: Vec<String>,
	pub pre_setup: Vec<String>,
	pub post_setup: Vec<String>,
}

impl Config {
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_file(path: &Path) -> Result<Self> {
		let mut config_file =
			File::open(path).map_err(|e| sys_error!("failed to open {}: {e}", path.display()))?;
		let mut content = String::new();
		config_file
			.read_to_string(&mut content)
			.map_err(|e| sys_error!("{e}"))?;

		Self::try_from(content.as_str())
	}

	/// Collects an array of strings from a YAML hash.
	fn collect_array(hash: &saphyr::Hash, key: &str) -> Result<Vec<String>> {
		match hash.get(&Yaml::from_str(key)) {
			Some(Yaml::Array(array)) => {
				let mut vec = Vec::new();
				for s in array {
					if let Yaml::String(s) = s {
						vec.push(s.clone());
					} else {
						return Err(config_error!(
							"expected element of {key} to be string, but found: {s:?}"
						));
					}
				}
				Ok(vec)
			}
			None => Ok(Vec::new()),
			_ => Err(config_error!("expected {key} to be an array")),
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

	pub fn list_apps(&self) {
		for app in &self.apps {
			println!("{}", app.name);
		}
	}

	pub fn backup(&self) -> Result<()> {
		let backup_dir = self
			.backup_dir
			.clone()
			.ok_or(config_error!("backup_dir not set"))?;
		let config_root =
			dirs::home_dir().ok_or(sys_error!("unknown system, cannot decide home directory"))?;

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
				let mut found = false;
				for app in &self.apps {
					if *selected == app.name {
						app.backup(&config_root, &backup_dir, self.clean, &self.ignore)?;
						found = true;
						break;
					}
				}

				if !found {
					return Err(arg_error!("app not found: {}", selected));
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

	pub fn setup(&self) -> Result<()> {
		let backup_dir = self
			.backup_dir
			.clone()
			.ok_or(config_error!("backup_dir not set"))?;
		let config_root =
			dirs::home_dir().ok_or(sys_error!("unknown system, cannot decide home directory"))?;

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
				let mut found = false;

				for app in &self.apps {
					if *selected == app.name {
						app.setup(&config_root, &backup_dir, self.clean, &self.ignore)?;
						found = true;
						break;
					}
				}

				if !found {
					return Err(arg_error!("app not found: {selected}"));
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
}

impl std::fmt::Display for Config {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(backup_dir) = &self.backup_dir {
			writeln!(f, "backup_dir: {}", backup_dir.display())?;
		}

		writeln!(f, "clean: {}", self.clean)?;

		write!(
			f,
			"{}",
			Self::format_array(
				"ignore",
				&self
					.ignore
					.iter()
					.map(std::string::ToString::to_string)
					.collect(),
				0
			)
		)?;

		if !self.selected_apps.is_empty() {
			writeln!(f, "# selected_apps: {:?}", self.selected_apps)?;
		}

		if !self.apps.is_empty() {
			writeln!(f, "apps:")?;
			for app in &self.apps {
				write!(f, "{app}")?;
			}
		}

		// TODO: pretty print multi-line string
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

impl TryFrom<&str> for Config {
	type Error = Error;
	fn try_from(value: &str) -> Result<Self> {
		let mut config = Self::new();
		let yaml = &Yaml::load_from_str(value).map_err(|e| config_error!("{e}"))?[0];

		if let Yaml::Hash(yaml) = yaml {
			match yaml.get(&Yaml::from_str("backup_dir")) {
				Some(Yaml::String(s)) => {
					config.backup_dir =
						Some(expanduser(s).map_err(|e| config_error!("expanduser error: {e}"))?);
				}
				None => (),
				_ => {
					return Err(config_error!("expected backup_dir to be a string"));
				}
			}

			match yaml.get(&Yaml::from_str("clean")) {
				Some(Yaml::Boolean(b)) => config.clean = *b,
				None => (),
				_ => {
					return Err(config_error!("expected clean to be a boolean"));
				}
			}

			for glob in Config::collect_array(yaml, "ignore")? {
				config
					.ignore
					.push(Pattern::new(&glob).map_err(|e| config_error!("invalid glob: {e}"))?);
			}

			match yaml.get(&Yaml::from_str("apps")) {
				Some(Yaml::Hash(app_hash)) => {
					for (name, value) in app_hash {
						if let Yaml::String(name) = name {
							let app = App::load_from_config(name, value)?;
							config.apps.push(app);
						} else {
							return Err(config_error!(
								"expected app name to be a string, but found: {name:?}"
							));
						}
					}
				}
				None => (),
				_ => {
					return Err(config_error!("expected apps to be a mapping"));
				}
			}

			config.pre_setup = Config::collect_array(yaml, "pre_setup")?;
			config.pre_backup = Config::collect_array(yaml, "pre_backup")?;
			config.post_backup = Config::collect_array(yaml, "post_backup")?;
			config.post_setup = Config::collect_array(yaml, "post_setup")?;

			Ok(config)
		} else {
			Err(config_error!("configuration not valid"))
		}
	}
}
