mod app;

pub use app::*;

use crate::{
	arg_error, config_error,
	error::{Error, Result},
	expanduser, run_hooks, sys_error,
};
use glob::Pattern;
use saphyr::{LoadableYamlNode, Yaml};
use std::{
	fs::File,
	io::Read,
	path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default)]
pub struct Config {
	// NOTE: CLI args may change these, be sure to consider them in `apply_file`
	pub selected_apps: Vec<String>,
	pub clean: bool,
	pub verbose: bool,

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

	/// Collects an array of strings from a YAML mapping.
	fn collect_array(mapping: &saphyr::Mapping, key: &str) -> Result<Vec<String>> {
		match mapping.get(&Yaml::value_from_str(key)) {
			Some(yaml) => {
				if let Some(array) = yaml.as_sequence() {
					let mut vec = Vec::new();
					for s in array {
						if let Some(s) = s.as_str() {
							vec.push(s.to_string());
						} else {
							return Err(config_error!(
								"expected element of {key} to be string, but found: {s:?}"
							));
						}
					}
					Ok(vec)
				} else {
					Err(config_error!("expected {key} to be an array"))
				}
			}
			None => Ok(Vec::new()),
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

	pub fn get_backup_dir(&self) -> Result<&PathBuf> {
		self.backup_dir
			.as_ref()
			.ok_or(config_error!("backup_dir not set"))
	}

	pub fn backup(&self) -> Result<()> {
		let backup_dir = self.get_backup_dir()?;

		run_hooks(&self.pre_backup, backup_dir, "pre-backup hooks")?;

		if self.selected_apps.is_empty() {
			for app in &self.apps {
				app.backup(self)?;
			}
		} else {
			for selected in &self.selected_apps {
				let mut found = false;
				for app in &self.apps {
					if *selected == app.name {
						app.backup(self)?;
						found = true;
						break;
					}
				}

				if !found {
					return Err(arg_error!("app not found: {}", selected));
				}
			}
		}

		run_hooks(&self.post_backup, backup_dir, "post-backup hooks")
	}

	pub fn setup(&self) -> Result<()> {
		let backup_dir = self.get_backup_dir()?;

		run_hooks(&self.pre_setup, backup_dir, "pre-setup hooks")?;

		if self.selected_apps.is_empty() {
			for app in &self.apps {
				app.setup(self)?;
			}
		} else {
			for selected in &self.selected_apps {
				let mut found = false;
				for app in &self.apps {
					if *selected == app.name {
						app.setup(self)?;
						found = true;
						break;
					}
				}

				if !found {
					return Err(arg_error!("app not found: {}", selected));
				}
			}
		}

		run_hooks(&self.post_setup, backup_dir, "post-setup hooks")
	}

	pub fn apply_file(&mut self, path: &Path) -> Result<()> {
		let config = Config::from_file(path)?;
		let clean = if self.clean { true } else { config.clean };

		*self = Self {
			verbose: self.verbose,
			selected_apps: self.selected_apps.clone(),
			clean,
			..config
		};

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

		if let Some(yaml) = yaml.as_mapping() {
			if let Some(backup_dir) = yaml.get(&Yaml::value_from_str("backup_dir")) {
				if let Some(s) = backup_dir.as_str() {
					config.backup_dir =
						Some(expanduser(s).map_err(|e| config_error!("expanduser error: {e}"))?);
				} else {
					return Err(config_error!("expected backup_dir to be a string"));
				}
			}

			if let Some(clean) = yaml.get(&Yaml::value_from_str("clean")) {
				if let Some(b) = clean.as_bool() {
					config.clean = b;
				} else {
					return Err(config_error!("expected clean to be a boolean"));
				}
			}

			for glob in Config::collect_array(yaml, "ignore")? {
				config
					.ignore
					.push(Pattern::new(&glob).map_err(|e| config_error!("invalid glob: {e}"))?);
			}

			if let Some(apps) = yaml.get(&Yaml::value_from_str("apps")) {
				if let Some(app_hash) = apps.as_mapping() {
					for (name, value) in app_hash {
						if let Some(name) = name.as_str() {
							let app = App::load_from_config(name, value)?;
							config.apps.push(app);
						} else {
							return Err(config_error!(
								"expected app name to be a string, but found: {name:?}"
							));
						}
					}
				} else {
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
