mod helper;

use dotbackup::cli::Config;
use helper::*;
use serial_test::serial;
use std::{env, fs, path::Path};

#[test]
fn test_empty() {
	let config = Config::try_from("");
	assert!(config.is_err());
}

#[test]
fn test_minimal() {
	let config = Config::try_from("backup_dir: ~/backup");
	assert!(config.is_ok());
}

#[test]
#[serial]
fn test_basic() {
	let config = Config::try_from(include_str!("configs/basic.yml")).unwrap();

	cleanup();
	write_file("test/.config/app_a/a1.txt", "a1");
	write_file("test/.config/app_a/a2.txt", "a2");
	write_file("test/.config/app_b/b1.txt", "b1");
	write_file("test/.config/app_b/b2.txt", "b2");
	write_file("test/.config/app_b/b3.txt", "b3");
	config.backup().unwrap();
	assert_eq!(
		"a1",
		fs::read_to_string("test/backup/.config/app_a/a1.txt").unwrap()
	);
	assert_eq!(
		"a2",
		fs::read_to_string("test/backup/.config/app_a/a2.txt").unwrap()
	);
	assert_eq!(
		"b1",
		fs::read_to_string("test/backup/.config/app_b/b1.txt").unwrap()
	);
	assert_eq!(
		"b2",
		fs::read_to_string("test/backup/.config/app_b/b2.txt").unwrap()
	);
	assert!(!Path::new("test/backup/.config/app_b/b3.txt").is_file());

	fs::remove_dir_all("test/.config").unwrap();
	write_file("test/backup/.config/app_b/b3.txt", "b3");
	config.setup().unwrap();
	assert_eq!(
		"a1",
		fs::read_to_string("test/.config/app_a/a1.txt").unwrap()
	);
	assert_eq!(
		"a2",
		fs::read_to_string("test/.config/app_a/a2.txt").unwrap()
	);
	assert_eq!(
		"b1",
		fs::read_to_string("test/.config/app_b/b1.txt").unwrap()
	);
	assert_eq!(
		"b2",
		fs::read_to_string("test/.config/app_b/b2.txt").unwrap()
	);
	assert!(!Path::new("test/.config/app_b/b3.txt").is_file());
}

#[test]
fn test_complex_script() {
	let config = Config::try_from(include_str!("configs/complex_script.yml")).unwrap();
	assert!(config.backup().is_ok());
}

#[test]
#[serial]
fn test_ignore() {
	let config = Config::try_from(include_str!("configs/ignore.yml")).unwrap();

	cleanup();
	write_file("test/.config/app/ignore/global_ignore", "global_ignore");
	write_file("test/.config/app/ignore/app_ignore", "app_ignore");
	write_file("test/.config/app/global_ignore", "global_ignore");
	write_file("test/.config/app/app_ignore", "app_ignore");
	config.backup().unwrap();
	assert!(!Path::new("test/backup/.config/app/ignore/global_ignore").is_file());
	assert!(!Path::new("test/backup/.config/app/ignore/app_ignore").is_file());
	assert!(Path::new("test/backup/.config/app/ignore").is_dir());
	assert_eq!(
		fs::read_to_string("test/backup/.config/app/global_ignore").unwrap(),
		"global_ignore"
	);
	assert_eq!(
		fs::read_to_string("test/backup/.config/app/app_ignore").unwrap(),
		"app_ignore"
	);

	fs::remove_dir_all("test/.config").unwrap();
	write_file(
		"test/backup/.config/app/ignore/global_ignore",
		"global_ignore",
	);
	write_file("test/backup/.config/app/ignore/app_ignore", "app_ignore");
	config.setup().unwrap();
	assert!(!Path::new("test/.config/app/ignore/global_ignore").is_file());
	assert!(!Path::new("test/.config/app/ignore/app_ignore").is_file());
	assert!(Path::new("test/.config/app/ignore").is_dir());
	assert_eq!(
		fs::read_to_string("test/.config/app/global_ignore").unwrap(),
		"global_ignore"
	);
	assert_eq!(
		fs::read_to_string("test/.config/app/app_ignore").unwrap(),
		"app_ignore"
	);
}

#[test]
fn test_unknown_option() {
	let config_a = Config::try_from(include_str!("configs/unknown_option.yml")).unwrap();
	let config_b = Config::try_from("backup_dir: test").unwrap();
	assert_eq!(config_a, config_b);
}

#[test]
#[serial]
fn test_os_files() {
	let config = Config::try_from(include_str!("configs/os_files.yml")).unwrap();

	cleanup();
	write_file("test/.config/app/all.txt", "all");
	write_file("test/.config/app/linux.txt", "linux");
	write_file("test/.config/app/macos.txt", "macos");
	write_file("test/.config/app/windows.txt", "windows");
	config.backup().unwrap();
	assert_eq!(
		fs::read_to_string("test/backup/.config/app/all.txt").unwrap(),
		"all"
	);
	if cfg!(target_os = "linux") {
		assert_eq!(
			fs::read_to_string("test/backup/.config/app/linux.txt").unwrap(),
			"linux"
		);
	} else {
		assert!(!Path::new("test/backup/.config/app/linux.txt").is_file());
	}
	if cfg!(target_os = "macos") {
		assert_eq!(
			fs::read_to_string("test/backup/.config/app/macos.txt").unwrap(),
			"macos"
		);
	} else {
		assert!(!Path::new("test/backup/.config/app/macos.txt").is_file());
	}
	if cfg!(target_os = "windows") {
		assert_eq!(
			fs::read_to_string("test/backup/.config/app/windows.txt").unwrap(),
			"windows"
		);
	} else {
		assert!(!Path::new("test/backup/.config/app/windows.txt").is_file());
	}

	fs::remove_dir_all("test/.config").unwrap();
	write_file("test/.config/backup/app/linux.txt", "linux");
	write_file("test/.config/backup/app/macos.txt", "macos");
	write_file("test/.config/backup/app/windows.txt", "windows");
	config.setup().unwrap();
	assert_eq!(
		fs::read_to_string("test/.config/app/all.txt").unwrap(),
		"all"
	);
	if cfg!(target_os = "linux") {
		assert_eq!(
			fs::read_to_string("test/.config/app/linux.txt").unwrap(),
			"linux"
		);
	} else {
		assert!(!Path::new("test/.config/app/linux.txt").is_file());
	}
	if cfg!(target_os = "macos") {
		assert_eq!(
			fs::read_to_string("test/.config/app/macos.txt").unwrap(),
			"macos"
		);
	} else {
		assert!(!Path::new("test/.config/app/macos.txt").is_file());
	}
	if cfg!(target_os = "windows") {
		assert_eq!(
			fs::read_to_string("test/.config/app/windows.txt").unwrap(),
			"windows"
		);
	} else {
		assert!(!Path::new("test/.config/app/windows.txt").is_file());
	}
}

#[test]
#[serial]
fn test_global_backup_dir() {
	let config = Config::try_from(include_str!("configs/global_backup_dir.yml")).unwrap();

	cleanup();
	write_file("test/.config/app.txt", "app");
	config.backup().unwrap();
	let path = format!(
		"test/backup{}/.config/app.txt",
		if cfg!(target_os = "linux") {
			"_linux"
		} else if cfg!(target_os = "macos") {
			"_macos"
		} else if cfg!(target_os = "windows") {
			"_windows"
		} else {
			""
		}
	);
	assert_eq!(fs::read_to_string(path).unwrap(), "app");

	fs::remove_dir_all("test/.config").unwrap();
	config.setup().unwrap();
	assert_eq!(fs::read_to_string("test/.config/app.txt").unwrap(), "app");
}

#[test]
#[serial]
fn test_app_backup_dir() {
	let config = Config::try_from(include_str!("configs/app_backup_dir.yml")).unwrap();

	cleanup();
	write_file("test/.config/app_a.txt", "a");
	write_file("test/.config/app_b.txt", "b");
	config.backup().unwrap();
	assert_eq!(
		fs::read_to_string("test/backup_a/.config/app_a.txt").unwrap(),
		"a"
	);
	let path = format!(
		"test/backup{}/.config/app_b.txt",
		if cfg!(target_os = "linux") {
			"_linux"
		} else if cfg!(target_os = "macos") {
			"_macos"
		} else if cfg!(target_os = "windows") {
			"_windows"
		} else {
			""
		}
	);
	assert_eq!(fs::read_to_string(path).unwrap(), "b");

	fs::remove_dir_all("test/.config").unwrap();
	config.setup().unwrap();
	assert_eq!(fs::read_to_string("test/.config/app_a.txt").unwrap(), "a");
	assert_eq!(fs::read_to_string("test/.config/app_b.txt").unwrap(), "b");
}

#[test]
#[serial]
fn test_expandhome() {
	let home = env::current_dir().unwrap().join("test");
	set_home(&home);

	let config = Config::try_from(include_str!("configs/expandhome.yml")).unwrap();
	assert_eq!(config.get_dotfile_root(), home);

	cleanup();
	write_file("test/.config/app.txt", "app");
	config.backup().unwrap();
	assert_eq!(
		fs::read_to_string("test/backup/.config/app.txt").unwrap(),
		"app"
	);

	fs::remove_dir_all("test/.config").unwrap();
	config.setup().unwrap();
	assert_eq!(fs::read_to_string("test/.config/app.txt").unwrap(), "app");
}
