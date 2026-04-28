use std::{fs, path::Path};

/// Make an empty directory "test"
///
/// Note: only cleanup before test so that we can inspect failed tests
pub fn cleanup() {
	let test = Path::new("test");
	if test.is_dir() {
		fs::remove_dir_all(test).unwrap();
	}
	fs::create_dir("test").unwrap();
}

/// Create and write text to file
pub fn write_file(path: &str, text: &str) {
	let path = Path::new(path);

	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent).unwrap();
	}

	fs::write(path, text).unwrap();
}
