use super::Config;
use crate::{
	Cli, arg_error, config_error, copy_dir_all, error::Result, info, log, run_hooks, sys_error,
	warn,
};
use expanduser::expanduser;
use glob::Pattern;
use saphyr::Yaml;
use std::{
	fmt::Display,
	fs::{self, create_dir_all, remove_dir_all, remove_file},
	path::PathBuf,
};

#[derive(Debug, Clone, Default)]
pub struct App {
	pub name: String,
	pub files: Vec<PathBuf>,
	pub ignore: Vec<Pattern>,
	pub pre_backup: Vec<String>,
	pub post_backup: Vec<String>,
	pub pre_setup: Vec<String>,
	pub post_setup: Vec<String>,
}

impl App {
	#[must_use]
	pub fn new(name: &str) -> Self {
		Self {
			name: name.to_string(),
			..Default::default()
		}
	}

	pub fn load_from_config(name: &str, config: &Yaml) -> Result<Self> {
		if let Yaml::Mapping(config) = config {
			let mut app = Self::new(name);

			for file in Config::collect_array(config, "files")? {
				app.files
					.push(expanduser(file).map_err(|e| arg_error!("expanduser error: {e}"))?);
			}

			for glob in Config::collect_array(config, "ignore")? {
				app.ignore
					.push(Pattern::new(&glob).map_err(|e| arg_error!("invalid glob: {e}"))?);
			}

			app.pre_backup = Config::collect_array(config, "pre_backup")?;
			app.post_backup = Config::collect_array(config, "post_backup")?;
			app.pre_setup = Config::collect_array(config, "pre_setup")?;
			app.post_setup = Config::collect_array(config, "post_setup")?;

			Ok(app)
		} else {
			Err(arg_error!("expected app {name} to a mapping"))
		}
	}

	#[allow(clippy::unnecessary_debug_formatting, clippy::missing_panics_doc)]
	pub fn backup(&self, config: &Config) -> Result<()> {
		let config_root = Cli::config_root()?;
		let backup_dir = config.get_backup_dir()?;

		run_hooks(
			&self.pre_backup,
			backup_dir,
			&format!("pre-backup hooks for {}", self.name),
		)?;

		info!(":: Starting backup for {}...", self.name);
		let mut ignore = self.ignore.clone();
		ignore.extend(config.ignore.clone());
		for src in &self.files {
			if !src.starts_with(&config_root) {
				return Err(config_error!(
					"expected files under the configuration root ({}): {}",
					config_root.display(),
					src.display()
				));
			}

			if !src.exists() {
				warn!("SKIP: file not found: {src:?}");
				continue;
			}

			let dest = backup_dir.join(src.strip_prefix(&config_root).unwrap());
			if let Some(dest_dir) = dest.parent() {
				if !dest_dir.exists() {
					log!(config.verbose, "MKDIR: {dest_dir:?}");
					create_dir_all(dest_dir)
						.map_err(|e| sys_error!("create directory error: {e}"))?;
				}
			}
			if config.clean && dest.exists() {
				log!(config.verbose, "CLEAN: remove {dest:?}");
				if dest.is_file() {
					remove_file(&dest).map_err(|e| sys_error!("remove file error: {e}"))?;
				} else {
					remove_dir_all(&dest).map_err(|e| sys_error!("remove directory error: {e}"))?;
				}
			}

			eprintln!("COPY: {src:?} -> {dest:?}");
			if src.is_file() {
				fs::copy(src, dest).map_err(|e| sys_error!("copy file error: {e}"))?;
			} else {
				copy_dir_all(src, dest, &ignore, config.verbose)
					.map_err(|e| sys_error!("copy directory error: {e}"))?;
			}
		}

		run_hooks(
			&self.post_backup,
			backup_dir,
			&format!("post-backup hooks for {}", self.name),
		)
	}

	#[allow(clippy::unnecessary_debug_formatting, clippy::missing_panics_doc)]
	pub fn setup(&self, config: &Config) -> Result<()> {
		let config_root = Cli::config_root()?;
		let backup_dir = config.get_backup_dir()?;

		run_hooks(
			&self.pre_setup,
			backup_dir,
			&format!("pre-setup hooks for {}", self.name),
		)?;

		info!(":: Starting setup for {}...", self.name);
		let mut ignore = self.ignore.clone();
		ignore.extend(config.ignore.clone());
		for dest in &self.files {
			if !dest.starts_with(&config_root) {
				return Err(config_error!(
					"configuration file not under configuration root: {}",
					dest.display()
				));
			}

			let src = backup_dir.join(dest.strip_prefix(&config_root).unwrap());
			if !src.exists() {
				warn!("SKIP: file not found: {src:?}");
				continue;
			}

			if let Some(dest_dir) = dest.parent() {
				if !dest_dir.exists() {
					log!(config.verbose, "MKDIR: {dest_dir:?}");
					create_dir_all(dest_dir)
						.map_err(|e| sys_error!("create directory error: {e}"))?;
				}
			}
			if config.clean && dest.exists() {
				log!(config.verbose, "CLEAN: remove {dest:?}");
				if dest.is_file() {
					remove_file(dest).map_err(|e| sys_error!("remove file error: {e}"))?;
				} else {
					remove_dir_all(dest).map_err(|e| sys_error!("remove directory error: {e}"))?;
				}
			}

			eprintln!("COPY: {src:?} -> {dest:?}");
			if src.is_file() {
				fs::copy(src, dest).map_err(|e| sys_error!("copy file error: {e}"))?;
			} else {
				copy_dir_all(src, dest, &ignore, config.verbose)
					.map_err(|e| sys_error!("copy directory error: {e}"))?;
			}
		}

		run_hooks(
			&self.post_setup,
			backup_dir,
			&format!("post-setup hooks for {}", self.name),
		)
	}
}

impl Display for App {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "  {}:", self.name)?;
		write!(
			f,
			"{}",
			Config::format_array(
				"files",
				&self
					.files
					.iter()
					.map(|p| p
						.to_str()
						.expect("file path should be UTF-8 encoded")
						.to_string())
					.collect(),
				2
			)
		)?;
		write!(
			f,
			"{}",
			Config::format_array(
				"ignore",
				&self
					.ignore
					.iter()
					.map(std::string::ToString::to_string)
					.collect(),
				2
			)
		)?;

		// TODO: pretty print multi-line string
		write!(
			f,
			"{}",
			Config::format_array("pre_backup", &self.pre_backup, 2)
		)?;
		write!(
			f,
			"{}",
			Config::format_array("post_backup", &self.post_backup, 2)
		)?;
		write!(
			f,
			"{}",
			Config::format_array("pre_setup", &self.pre_setup, 2)
		)?;
		write!(
			f,
			"{}",
			Config::format_array("post_setup", &self.post_setup, 2)
		)
	}
}
