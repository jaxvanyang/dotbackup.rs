mod app;

pub use app::*;

use crate::{
	arg_error, config_error,
	error::{Error, Result},
	expanduser, info, run_hooks, sys_error,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	fs::File,
	io::Read,
	path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
	// NOTE: CLI args may change these, be sure to consider them in `apply_file`
	//
	// empty means select all apps
	#[serde(skip)]
	pub selected_apps: Vec<String>,
	#[serde(default)]
	pub clean: bool,
	#[serde(default)]
	pub verbose: bool,

	pub backup_dir: PathBuf,
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub ignore: Vec<String>,
	#[serde(default)]
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub apps: BTreeMap<String, App>,
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub pre_backup: Vec<String>,
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub post_backup: Vec<String>,
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub pre_setup: Vec<String>,
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
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

	pub fn expanduser(&mut self) {
		self.backup_dir = expanduser(&self.backup_dir);
		for app in self.apps.values_mut() {
			for file in &mut app.files {
				*file = expanduser(file);
			}
		}
	}

	#[must_use]
	pub fn get_selected_apps(&self) -> Vec<String> {
		if self.selected_apps.is_empty() {
			self.apps.keys().cloned().collect()
		} else {
			self.selected_apps.clone()
		}
	}

	pub fn list_apps(&self) {
		for name in self.apps.keys() {
			println!("{name}");
		}
	}

	pub fn backup(&self) -> Result<()> {
		let backup_dir = &self.backup_dir;
		let selected_apps = self.get_selected_apps();

		run_hooks(&self.pre_backup, backup_dir, "pre-backup hooks")?;

		for name in &selected_apps {
			if !self.apps.contains_key(name) {
				return Err(arg_error!("app not found: {}", name));
			}
			let app = &self.apps[name];

			run_hooks(
				&app.pre_backup,
				backup_dir,
				&format!("pre-backup hooks for {name}"),
			)?;
			info!("Starting backup for {name}...");

			self.apps[name].backup(self)?;

			run_hooks(
				&app.post_backup,
				backup_dir,
				&format!("post-backup hooks for {name}"),
			)?;
		}

		run_hooks(&self.post_backup, backup_dir, "post-backup hooks")
	}

	pub fn setup(&self) -> Result<()> {
		let backup_dir = &self.backup_dir;
		let selected_apps = self.get_selected_apps();

		run_hooks(&self.pre_setup, backup_dir, "pre-setup hooks")?;

		for name in &selected_apps {
			if !self.apps.contains_key(name) {
				return Err(arg_error!("app not found: {}", name));
			}
			let app = &self.apps[name];

			run_hooks(
				&app.pre_setup,
				backup_dir,
				&format!("pre-setup hooks for {name}"),
			)?;
			info!("Starting setup for {name}...");

			self.apps[name].setup(self)?;

			run_hooks(
				&app.post_setup,
				backup_dir,
				&format!("post-setup hooks for {name}"),
			)?;
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
		if !self.selected_apps.is_empty() {
			writeln!(f, "# selected_apps: {:?}", self.selected_apps)?;
		}

		writeln!(
			f,
			"{}",
			yaml_serde::to_string(self).map_err(|_| std::fmt::Error)?
		)
	}
}

impl TryFrom<&str> for Config {
	type Error = Error;
	fn try_from(value: &str) -> Result<Self> {
		let mut config: Self = yaml_serde::from_str(value).map_err(|e| config_error!("{e}"))?;
		config.expanduser();
		Ok(config)
	}
}
