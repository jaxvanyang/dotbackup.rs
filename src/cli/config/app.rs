use super::Config;
use crate::{arg_error, config_error, copy_dir_all, error::Result, log, sys_error, warn};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::{
	fmt::Display,
	fs::{self, create_dir_all, remove_dir_all, remove_file},
	path::PathBuf,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct App {
	// pub name: String,
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub files: Vec<PathBuf>,
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub ignore: Vec<String>,
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

impl App {
	fn merge_patterns(local: &Vec<String>, global: &Vec<String>) -> Result<Vec<Pattern>> {
		let mut ret = Vec::new();
		for s in local {
			ret.push(Pattern::new(s).map_err(|e| arg_error!("invalid glob pattern: {e:?}"))?);
		}
		for s in global {
			ret.push(Pattern::new(s).map_err(|e| arg_error!("invalid glob pattern: {e:?}"))?);
		}

		Ok(ret)
	}

	#[allow(clippy::unnecessary_debug_formatting, clippy::missing_panics_doc)]
	pub fn backup(&self, config: &Config) -> Result<()> {
		let dotfile_root = config.get_dotfile_root();
		let backup_dir = &config.backup_dir;
		let ignore = App::merge_patterns(&self.ignore, &config.ignore)?;

		for src in &self.files {
			if !src.starts_with(&dotfile_root) {
				return Err(config_error!(
					"the file ({}) is expected to be under the dotfile root ({})",
					src.display(),
					dotfile_root.display(),
				));
			}

			if !src.exists() {
				warn!("LOG: skip: file not found: {src:?}");
				continue;
			}

			let dest = backup_dir.join(src.strip_prefix(&dotfile_root).unwrap());
			if let Some(dest_dir) = dest.parent()
				&& !dest_dir.exists()
			{
				log!(config.verbose, "LOG: mkdir {dest_dir:?}");
				create_dir_all(dest_dir).map_err(|e| sys_error!("create directory error: {e}"))?;
			}
			if config.clean && dest.exists() {
				log!(config.verbose, "LOG: clean: remove {dest:?}");
				if dest.is_file() {
					remove_file(&dest).map_err(|e| sys_error!("remove file error: {e}"))?;
				} else {
					remove_dir_all(&dest).map_err(|e| sys_error!("remove directory error: {e}"))?;
				}
			}

			eprintln!("  {src:?} -> {dest:?}");
			if src.is_file() {
				fs::copy(src, dest).map_err(|e| sys_error!("copy file error: {e}"))?;
			} else {
				copy_dir_all(src, dest, &ignore, config.verbose)
					.map_err(|e| sys_error!("copy directory error: {e}"))?;
			}
		}

		Ok(())
	}

	#[allow(clippy::unnecessary_debug_formatting, clippy::missing_panics_doc)]
	pub fn setup(&self, config: &Config) -> Result<()> {
		let dotfile_root = config.get_dotfile_root();
		let backup_dir = &config.backup_dir;
		let ignore = App::merge_patterns(&self.ignore, &config.ignore)?;

		for dest in &self.files {
			if !dest.starts_with(&dotfile_root) {
				return Err(config_error!(
					"the file ({}) is expected to be under the dotfile root ({})",
					dest.display(),
					dotfile_root.display(),
				));
			}

			let src = backup_dir.join(dest.strip_prefix(&dotfile_root).unwrap());
			if !src.exists() {
				warn!("LOG: skip: file not found: {src:?}");
				continue;
			}

			if let Some(dest_dir) = dest.parent()
				&& !dest_dir.exists()
			{
				log!(config.verbose, "LOG: mkdir: {dest_dir:?}");
				create_dir_all(dest_dir).map_err(|e| sys_error!("create directory error: {e}"))?;
			}
			if config.clean && dest.exists() {
				log!(config.verbose, "LOG: clean: remove {dest:?}");
				if dest.is_file() {
					remove_file(dest).map_err(|e| sys_error!("remove file error: {e}"))?;
				} else {
					remove_dir_all(dest).map_err(|e| sys_error!("remove directory error: {e}"))?;
				}
			}

			eprintln!("  {src:?} -> {dest:?}");
			if src.is_file() {
				fs::copy(src, dest).map_err(|e| sys_error!("copy file error: {e}"))?;
			} else {
				copy_dir_all(src, dest, &ignore, config.verbose)
					.map_err(|e| sys_error!("copy directory error: {e}"))?;
			}
		}

		Ok(())
	}
}

impl Display for App {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(
			f,
			"{}",
			yaml_serde::to_string(self).map_err(|_| std::fmt::Error)?
		)
	}
}
