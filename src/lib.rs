use std::{
	fmt::Display,
	fs::{self, create_dir_all},
	io::{self, Write},
	path::{Path, PathBuf},
	process::{Command, Stdio},
};

use glob::Pattern;

pub mod cli;
pub mod config;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum AppError {
	ConfigError(String),
	ArgError(String),
	SysError(String),
	AppError(String),
}

impl AppError {
	pub fn config_error(msg: &str) -> Self {
		Self::ConfigError(msg.to_string())
	}

	pub fn arg_error(msg: &str) -> Self {
		Self::ArgError(msg.to_string())
	}

	pub fn sys_error(msg: &str) -> Self {
		Self::SysError(msg.to_string())
	}

	pub fn new(msg: &str) -> Self {
		Self::AppError(msg.to_string())
	}
}

impl Display for AppError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ConfigError(s) => write!(f, "configuration error: {}", s),
			Self::ArgError(s) => write!(f, "CLI argument error: {}", s),
			Self::SysError(s) => write!(f, "system error: {}", s),
			Self::AppError(s) => write!(f, "app error: {}", s),
		}
	}
}

#[derive(Debug)]
pub enum CLIOpt {
	Backup,
	Setup,
	Help,
	List,
	Version,
	DumpConfig,
}

fn copy_dir_all(
	from: impl AsRef<Path>,
	to: impl AsRef<Path>,
	ignore: &Vec<Pattern>,
) -> io::Result<()> {
	create_dir_all(&to)?;
	for entry in from.as_ref().read_dir()? {
		let entry = entry?;
		let path = entry.path();
		for pattern in ignore {
			if pattern.matches_path(&PathBuf::from(path.file_name().unwrap())) {
				eprintln!("LOG: ignore {path:?}");
				return Ok(());
			}
		}
		if entry.file_type()?.is_dir() {
			copy_dir_all(entry.path(), to.as_ref().join(entry.file_name()), ignore)?;
		} else {
			fs::copy(entry.path(), to.as_ref().join(entry.file_name()))?;
		}
	}
	Ok(())
}

fn run_hook(script: &str, backup_dir: &Path) -> Result<(), AppError> {
	let script = format!("set -ex\n{script}");
	let mut sh = Command::new("sh")
		.arg("-s")
		.env("BACKUP_DIR", backup_dir.to_str().unwrap())
		.stdin(Stdio::piped())
		.spawn()
		.map_err(|e| AppError::SysError(format!("failed to spawn sh: {e}")))?;
	sh.stdin
		.take()
		.ok_or(AppError::sys_error("failed to open stdin of sh"))?
		.write_all(script.as_bytes())
		.map_err(|e| AppError::SysError(format!("failed to write stdin of sh: {e}")))?;
	let status = sh.wait().map_err(|e| AppError::SysError(e.to_string()))?;
	if status.success() {
		Ok(())
	} else {
		Err(AppError::SysError(format!(
			"sh returned non-zero: {}",
			status.code().unwrap()
		)))
	}
}
