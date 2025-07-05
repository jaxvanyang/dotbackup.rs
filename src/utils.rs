#[macro_use]
pub mod log;

use crate::{error::Result, sys_error};
use glob::Pattern;
use std::{
	fs,
	io::{self, Write},
	path::{Path, PathBuf},
	process::{Command, Stdio},
};

#[allow(clippy::missing_panics_doc)]
pub fn copy_dir_all(
	from: impl AsRef<Path>,
	to: impl AsRef<Path>,
	ignore: &Vec<Pattern>,
	verbose: bool,
) -> io::Result<()> {
	fs::create_dir_all(&to)?;

	for entry in from.as_ref().read_dir()? {
		let entry = entry?;
		let path = entry.path();

		for pattern in ignore {
			if pattern.matches_path(&PathBuf::from(path.file_name().unwrap())) {
				log!(verbose, "LOG: ignore {}", path.display());
				return Ok(());
			}
		}

		if entry.file_type()?.is_dir() {
			copy_dir_all(
				entry.path(),
				to.as_ref().join(entry.file_name()),
				ignore,
				verbose,
			)?;
		} else {
			fs::copy(entry.path(), to.as_ref().join(entry.file_name()))?;
		}
	}
	Ok(())
}

pub fn run_hook(script: &str, backup_dir: &Path) -> Result<()> {
	// TODO: Windows implementation
	let script = format!("set -ex\n{script}");
	let mut sh = Command::new("sh")
		.arg("-s")
		.env("BACKUP_DIR", backup_dir)
		.stdin(Stdio::piped())
		.spawn()
		.map_err(|e| sys_error!("failed to spawn sh: {e}"))?;

	sh.stdin
		.take()
		.ok_or(sys_error!("failed to open stdin of sh"))?
		.write_all(script.as_bytes())
		.map_err(|e| sys_error!("failed to write stdin of sh: {e}"))?;

	let status = sh.wait().map_err(|e| sys_error!("{e}"))?;

	if status.success() {
		Ok(())
	} else {
		unsafe {
			Err(sys_error!(
				"sh returned non-zero: {}",
				status.code().unwrap_unchecked()
			))
		}
	}
}

pub fn run_hooks(hooks: &[String], backup_dir: &Path, name: &str) -> Result<()> {
	let n = hooks.len();
	for (i, hook) in hooks.iter().enumerate() {
		info!(":: Running {name} ({}/{n})...", i + 1);
		run_hook(hook, backup_dir)?;
	}

	Ok(())
}
