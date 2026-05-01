mod app;

pub use app::*;
use dirs::home_dir;

use crate::{
	arg_error, config_error,
	consts::colors::{GREEN, RESET},
	error::{Error, Result},
	expandhome, run_hooks, sys_error,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	fs::File,
	io::Read,
	path::{Path, PathBuf},
};

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(value: &bool) -> bool {
	!value
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
	// NOTE: CLI args may change these, be sure to consider them in `apply_file`
	//
	// empty means select all apps
	#[serde(skip)]
	pub selected_apps: Vec<String>,
	#[serde(default)]
	#[serde(skip_serializing_if = "is_false")]
	pub clean: bool,
	#[serde(default)]
	#[serde(skip_serializing_if = "is_false")]
	pub verbose: bool,

	/// dotfile root directory, default is the home directory
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dotfile_root: Option<PathBuf>,

	pub backup_dir: PathBuf,

	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub backup_dir_linux: Option<PathBuf>,

	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub backup_dir_macos: Option<PathBuf>,

	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub backup_dir_windows: Option<PathBuf>,

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

	/// Return expanded `backup_dir`
	#[must_use]
	pub fn get_backup_dir(&self) -> PathBuf {
		let path = if cfg!(target_os = "linux")
			&& let Some(path) = self.backup_dir_linux.as_ref()
		{
			path
		} else if cfg!(target_os = "macos")
			&& let Some(path) = self.backup_dir_macos.as_ref()
		{
			path
		} else if cfg!(target_os = "windows")
			&& let Some(path) = self.backup_dir_windows.as_ref()
		{
			path
		} else {
			&self.backup_dir
		};

		expandhome(path)
	}

	#[must_use]
	pub fn get_selected_apps(&self) -> Vec<String> {
		if self.selected_apps.is_empty() {
			self.apps.keys().cloned().collect()
		} else {
			self.selected_apps.clone()
		}
	}

	/// Return expanded `dotfile_root`
	///
	/// # Panics
	///
	/// Will panic if home directory is unknown
	#[must_use]
	pub fn get_dotfile_root(&self) -> PathBuf {
		self.dotfile_root
			.as_ref()
			.map_or(home_dir().expect("home directory is unknown"), expandhome)
	}

	pub fn list_apps(&self) {
		for name in self.apps.keys() {
			println!("{name}");
		}
	}

	pub fn backup(&self) -> Result<()> {
		let backup_dir = &self.get_backup_dir();
		let selected_apps = self.get_selected_apps();

		run_hooks(&self.pre_backup, backup_dir, "pre-backup hooks")?;

		for name in &selected_apps {
			if !self.apps.contains_key(name) {
				return Err(arg_error!("app not found: {}", name));
			}
			let app = &self.apps[name];
			let highlight_name = format!("{GREEN}{name}{RESET}");

			run_hooks(
				&app.pre_backup,
				backup_dir,
				&format!("pre-backup hooks for {highlight_name}"),
			)?;

			self.apps[name].backup(name, self)?;

			run_hooks(
				&app.post_backup,
				backup_dir,
				&format!("post-backup hooks for {highlight_name}"),
			)?;
		}

		run_hooks(&self.post_backup, backup_dir, "post-backup hooks")
	}

	pub fn setup(&self) -> Result<()> {
		let backup_dir = &self.get_backup_dir();
		let selected_apps = self.get_selected_apps();

		run_hooks(&self.pre_setup, backup_dir, "pre-setup hooks")?;

		for name in &selected_apps {
			if !self.apps.contains_key(name) {
				return Err(arg_error!("app not found: {}", name));
			}
			let app = &self.apps[name];
			let highlight_name = format!("{GREEN}{name}{RESET}");

			run_hooks(
				&app.pre_setup,
				backup_dir,
				&format!("pre-setup hooks for {highlight_name}"),
			)?;

			self.apps[name].setup(name, self)?;

			run_hooks(
				&app.post_setup,
				backup_dir,
				&format!("post-setup hooks for {highlight_name}"),
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
		let config: Self = yaml_serde::from_str(value).map_err(|e| config_error!("{e}"))?;
		Ok(config)
	}
}
