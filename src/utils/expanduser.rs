use dirs::home_dir;
use std::path::{Path, PathBuf};

/// # Panics
///
/// Will panic if home directory is unknown
#[must_use]
pub fn expanduser<T: AsRef<Path>>(path: &T) -> PathBuf {
	let mut ret = path.as_ref().to_path_buf();

	if ret.starts_with("~") {
		let home = home_dir().expect("home directory is unknown");
		ret = home.join(ret.strip_prefix("~").unwrap());
	}

	ret
}
