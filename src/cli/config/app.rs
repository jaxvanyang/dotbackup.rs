use super::Config;
use crate::{
	arg_error, config_error, copy_dir_all, error::Result, expandhome, log, sys_error, warn,
};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::{
	fmt::Display,
	fs::{self, create_dir_all, remove_dir_all, remove_file},
	path::PathBuf,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct App {
	#[serde(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub backup_dir: Option<PathBuf>,

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
	pub files: Vec<PathBuf>,

	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub files_linux: Vec<PathBuf>,

	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub files_macos: Vec<PathBuf>,

	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub files_windows: Vec<PathBuf>,

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

	/// Return expanded app-level `backup_dir`
	#[must_use]
	pub fn get_app_backup_dir(&self) -> Option<PathBuf> {
		let path = if cfg!(target_os = "linux") && self.backup_dir_linux.is_some() {
			&self.backup_dir_linux
		} else if cfg!(target_os = "macos") && self.backup_dir_macos.is_some() {
			&self.backup_dir_macos
		} else if cfg!(target_os = "windows") && self.backup_dir_windows.is_some() {
			&self.backup_dir_windows
		} else {
			&self.backup_dir
		};

		path.as_ref().map(expandhome)
	}

	/// Return expanded `backup_dir`
	#[must_use]
	pub fn get_backup_dir(&self, config: &Config) -> PathBuf {
		self.get_app_backup_dir().unwrap_or(config.get_backup_dir())
	}

	/// Return all files to be backed up, including OS-specific files
	#[must_use]
	pub fn get_files(&self) -> Vec<PathBuf> {
		let mut ret = self.files.clone();
		if cfg!(target_os = "linux") {
			ret.extend(self.files_linux.clone());
		} else if cfg!(target_os = "macos") {
			ret.extend(self.files_macos.clone());
		} else if cfg!(target_os = "windows") {
			ret.extend(self.files_windows.clone());
		}

		ret
	}

	#[allow(clippy::missing_panics_doc)]
	pub fn backup(&self, config: &Config) -> Result<()> {
		let files = self.get_files();
		if files.is_empty() {
			log!(config.verbose, "no file should be copy");
			return Ok(());
		}

		let dotfile_root = config.get_dotfile_root();
		let backup_dir = self.get_backup_dir(config);
		let ignore = App::merge_patterns(&self.ignore, &config.ignore)?;

		for src in &files {
			let src = expandhome(src);
			if !src.starts_with(&dotfile_root) {
				return Err(config_error!(
					"the file ({}) is expected to be under the dotfile root ({})",
					src.display(),
					dotfile_root.display(),
				));
			}

			if !src.exists() {
				warn!("skip: file not found: {}", src.display());
				continue;
			}

			let dest = backup_dir.join(src.strip_prefix(&dotfile_root).unwrap());
			if let Some(dest_dir) = dest.parent()
				&& !dest_dir.exists()
			{
				log!(config.verbose, "mkdir {}", dest_dir.display());
				create_dir_all(dest_dir).map_err(|e| sys_error!("create directory error: {e}"))?;
			}
			if config.clean && dest.exists() {
				log!(config.verbose, "clean: remove {}", dest.display());
				if dest.is_file() {
					remove_file(&dest).map_err(|e| sys_error!("remove file error: {e}"))?;
				} else {
					remove_dir_all(&dest).map_err(|e| sys_error!("remove directory error: {e}"))?;
				}
			}

			eprintln!("  {} -> {}", src.display(), dest.display());
			if src.is_file() {
				fs::copy(src, dest).map_err(|e| sys_error!("copy file error: {e}"))?;
			} else {
				copy_dir_all(src, dest, &ignore, config.verbose)
					.map_err(|e| sys_error!("copy directory error: {e}"))?;
			}
		}

		Ok(())
	}

	#[allow(clippy::missing_panics_doc)]
	pub fn setup(&self, config: &Config) -> Result<()> {
		let files = self.get_files();
		if files.is_empty() {
			log!(config.verbose, "no file should be copy");
			return Ok(());
		}

		let dotfile_root = config.get_dotfile_root();
		let backup_dir = self.get_backup_dir(config);
		let ignore = App::merge_patterns(&self.ignore, &config.ignore)?;

		for dest in &files {
			let dest = expandhome(dest);
			if !dest.starts_with(&dotfile_root) {
				return Err(config_error!(
					"the file ({}) is expected to be under the dotfile root ({})",
					dest.display(),
					dotfile_root.display(),
				));
			}

			let src = backup_dir.join(dest.strip_prefix(&dotfile_root).unwrap());
			if !src.exists() {
				warn!("skip: file not found: {}", src.display());
				continue;
			}

			if let Some(dest_dir) = dest.parent()
				&& !dest_dir.exists()
			{
				log!(config.verbose, "mkdir: {}", dest_dir.display());
				create_dir_all(dest_dir).map_err(|e| sys_error!("create directory error: {e}"))?;
			}
			if config.clean && dest.exists() {
				log!(config.verbose, "clean: remove {}", dest.display());
				if dest.is_file() {
					remove_file(&dest).map_err(|e| sys_error!("remove file error: {e}"))?;
				} else {
					remove_dir_all(&dest).map_err(|e| sys_error!("remove directory error: {e}"))?;
				}
			}

			eprintln!("  {} -> {}", src.display(), dest.display());
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
