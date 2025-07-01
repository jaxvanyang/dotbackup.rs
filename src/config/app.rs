use std::{
	fmt::Display,
	fs::{self, create_dir_all, remove_dir_all, remove_file},
	path::{Path, PathBuf},
};

use expanduser::expanduser;
use glob::Pattern;
use saphyr::Yaml;

use crate::{AppError, copy_dir_all, run_hook};

use super::Config;

#[derive(Debug)]
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
	pub fn load_from_config(name: &str, config: &Yaml) -> Result<Self, AppError> {
		if let Yaml::Hash(config) = config {
			let files = Config::collect_array(config, "files")?
				.iter()
				.map(|s| expanduser(s).expect("file should be a path"))
				.collect();

			let mut ignore = Vec::new();
			for glob in Config::collect_array(config, "ignore")? {
				ignore.push(
					Pattern::new(&glob)
						.map_err(|e| AppError::ConfigError(format!("invalid glob: {e}")))?,
				);
			}

			let pre_backup = Config::collect_array(config, "pre_backup")?;
			let post_backup = Config::collect_array(config, "post_backup")?;
			let pre_setup = Config::collect_array(config, "pre_setup")?;
			let post_setup = Config::collect_array(config, "post_setup")?;

			Ok(Self {
				name: name.to_string(),
				files,
				ignore,
				pre_backup,
				post_backup,
				pre_setup,
				post_setup,
			})
		} else {
			Err(AppError::ConfigError(
				"app value should be a mapping".to_string(),
			))
		}
	}

	pub fn backup(
		&self,
		config_root: &Path,
		backup_dir: &Path,
		clean: bool,
		global_ignore: &[Pattern],
	) -> Result<(), AppError> {
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
				return Err(AppError::ConfigError(format!(
					"configuration file not under configuration root: {}",
					src.to_str().unwrap()
				)));
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
						.map_err(|e| AppError::SysError(format!("create directory error: {e}")))?;
				}
			}
			if clean && dest.exists() {
				println!("CLEAN: remove {dest:?}");
				if dest.is_file() {
					remove_file(&dest)
						.map_err(|e| AppError::SysError(format!("remove file error: {e}")))?;
				} else {
					remove_dir_all(&dest)
						.map_err(|e| AppError::SysError(format!("remove directory error: {e}")))?;
				}
			}

			println!("COPY: {src:?} -> {dest:?}");
			if src.is_file() {
				fs::copy(src, dest)
					.map_err(|e| AppError::SysError(format!("copy file error: {e}")))?;
			} else {
				copy_dir_all(src, dest, &ignore)
					.map_err(|e| AppError::SysError(format!("copy directory error: {e}")))?;
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

	pub fn setup(
		&self,
		config_root: &Path,
		backup_dir: &Path,
		clean: bool,
		global_ignore: &[Pattern],
	) -> Result<(), AppError> {
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
				return Err(AppError::ConfigError(format!(
					"configuration file not under configuration root: {}",
					dest.to_str().unwrap()
				)));
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
						.map_err(|e| AppError::SysError(format!("create directory error: {e}")))?;
				}
			}
			if clean && dest.exists() {
				println!("CLEAN: remove {dest:?}");
				if dest.is_file() {
					remove_file(dest)
						.map_err(|e| AppError::SysError(format!("remove file error: {e}")))?;
				} else {
					remove_dir_all(dest)
						.map_err(|e| AppError::SysError(format!("remove directory error: {e}")))?;
				}
			}

			println!("COPY: {src:?} -> {dest:?}");
			if src.is_file() {
				fs::copy(src, dest)
					.map_err(|e| AppError::SysError(format!("copy file error: {e}")))?;
			} else {
				copy_dir_all(src, dest, &ignore)
					.map_err(|e| AppError::SysError(format!("copy directory error: {e}")))?;
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
