use dirs::home_dir;
use std::path::PathBuf;

use crate::{error::Result, sys_error};

pub fn expanduser<S: AsRef<str>>(path: S) -> Result<PathBuf> {
	let home = home_dir().ok_or(sys_error!("unknown system, cannot decide home directory"))?;
	if path.as_ref() == "~" {
		return Ok(home);
	}

	Ok(if path.as_ref().starts_with("~/") {
		home.join(PathBuf::from(path.as_ref().replace("~/", "")))
	} else {
		PathBuf::from(path.as_ref())
	})
}
