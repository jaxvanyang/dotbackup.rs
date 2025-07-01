use super::Config;
use crate::{arg_error, config_error, copy_dir_all, error::Result, run_hook, sys_error};
use expanduser::expanduser;
use glob::Pattern;
use saphyr::Yaml;
use std::{
	fmt::Display,
	fs::{self, create_dir_all, remove_dir_all, remove_file},
	path::{Path, PathBuf},
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
		if let Yaml::Hash(config) = config {
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
	pub fn backup(
		&self,
		config_root: &Path,
		backup_dir: &Path,
		clean: bool,
		global_ignore: &[Pattern],
	) -> Result<()> {
		let n = self.pre_backup.len();
		for (i, hook) in self.pre_backup.iter().enumerate() {
			println!(
				":: Running pre-backup hooks for {} ({}/{n})...",
				self.name,
				i + 1
			);
			run_hook(hook, backup_dir)?;
		}

		println!(":: Starting backup for {}...", self.name);
		let mut ignore = self.ignore.clone();
		ignore.extend(global_ignore.to_owned());
		for src in &self.files {
			if !src.starts_with(config_root) {
				return Err(config_error!(
					"configuration file not under configuration root: {}",
					src.to_str().unwrap()
				));
			}

			if !src.exists() {
				println!("SKIP: file not found: {src:?}");
				continue;
			}

			let dest = backup_dir.join(src.strip_prefix(config_root).unwrap());
			if let Some(dest_dir) = dest.parent() {
				if !dest_dir.exists() {
					println!("MKDIR: {dest_dir:?}");
					create_dir_all(dest_dir)
						.map_err(|e| sys_error!("create directory error: {e}"))?;
				}
			}
			if clean && dest.exists() {
				println!("CLEAN: remove {dest:?}");
				if dest.is_file() {
					remove_file(&dest).map_err(|e| sys_error!("remove file error: {e}"))?;
				} else {
					remove_dir_all(&dest).map_err(|e| sys_error!("remove directory error: {e}"))?;
				}
			}

			println!("COPY: {src:?} -> {dest:?}");
			if src.is_file() {
				fs::copy(src, dest).map_err(|e| sys_error!("copy file error: {e}"))?;
			} else {
				copy_dir_all(src, dest, &ignore)
					.map_err(|e| sys_error!("copy directory error: {e}"))?;
			}
		}

		let n = self.post_backup.len();
		for (i, hook) in self.post_backup.iter().enumerate() {
			println!(
				":: Running post-backup hooks for {} ({}/{n})...",
				self.name,
				i + 1
			);
			run_hook(hook, backup_dir)?;
		}

		Ok(())
	}

	#[allow(clippy::unnecessary_debug_formatting, clippy::missing_panics_doc)]
	pub fn setup(
		&self,
		config_root: &Path,
		backup_dir: &Path,
		clean: bool,
		global_ignore: &[Pattern],
	) -> Result<()> {
		let n = self.pre_setup.len();
		for (i, hook) in self.pre_setup.iter().enumerate() {
			println!(
				":: Running pre-setup hooks for {} ({}/{n})...",
				self.name,
				i + 1
			);
			run_hook(hook, backup_dir)?;
		}

		println!(":: Starting setup for {}...", self.name);
		let mut ignore = self.ignore.clone();
		ignore.extend(global_ignore.to_owned());
		for dest in &self.files {
			if !dest.starts_with(config_root) {
				return Err(config_error!(
					"configuration file not under configuration root: {}",
					dest.display()
				));
			}

			let src = backup_dir.join(dest.strip_prefix(config_root).unwrap());
			if !src.exists() {
				println!("SKIP: file not found: {src:?}");
				continue;
			}

			if let Some(dest_dir) = dest.parent() {
				if !dest_dir.exists() {
					println!("MKDIR: {dest_dir:?}");
					create_dir_all(dest_dir)
						.map_err(|e| sys_error!("create directory error: {e}"))?;
				}
			}
			if clean && dest.exists() {
				println!("CLEAN: remove {dest:?}");
				if dest.is_file() {
					remove_file(dest).map_err(|e| sys_error!("remove file error: {e}"))?;
				} else {
					remove_dir_all(dest).map_err(|e| sys_error!("remove directory error: {e}"))?;
				}
			}

			println!("COPY: {src:?} -> {dest:?}");
			if src.is_file() {
				fs::copy(src, dest).map_err(|e| sys_error!("copy file error: {e}"))?;
			} else {
				copy_dir_all(src, dest, &ignore)
					.map_err(|e| sys_error!("copy directory error: {e}"))?;
			}
		}

		let n = self.post_setup.len();
		for (i, hook) in self.post_setup.iter().enumerate() {
			println!(
				":: Running post-setup hooks for {} ({}/{n})...",
				self.name,
				i + 1
			);
			run_hook(hook, backup_dir)?;
		}

		Ok(())
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
