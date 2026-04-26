mod helper;

use dotbackup::cli::Config;
use helper::*;
use std::{fs, path::Path};
use serial_test::serial;


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
	assert_eq!(
		config,
		Config::try_from(config.to_string().as_str()).unwrap()
	);

	cleanup();
	write_file("test/.config/app_a/a1.txt", "a1");
	write_file("test/.config/app_a/a2.txt", "a2");
	write_file("test/.config/app_b/b1.txt", "b1");
	write_file("test/.config/app_b/b2.txt", "b2");
	write_file("test/.config/app_b/b3.txt", "b3");
	config.backup().unwrap();
	assert_eq!("a1", fs::read_to_string("test/backup/.config/app_a/a1.txt").unwrap());
	assert_eq!("a2", fs::read_to_string("test/backup/.config/app_a/a2.txt").unwrap());
	assert_eq!("b1", fs::read_to_string("test/backup/.config/app_b/b1.txt").unwrap());
	assert_eq!("b2", fs::read_to_string("test/backup/.config/app_b/b2.txt").unwrap());
	assert!(!Path::new("test/backup/.config/app_b/b3.txt").is_file());

	fs::remove_dir_all("test/.config").unwrap();
	write_file("test/backup/.config/app_b/b3.txt", "b3");
	config.setup().unwrap();
	assert_eq!("a1", fs::read_to_string("test/.config/app_a/a1.txt").unwrap());
	assert_eq!("a2", fs::read_to_string("test/.config/app_a/a2.txt").unwrap());
	assert_eq!("b1", fs::read_to_string("test/.config/app_b/b1.txt").unwrap());
	assert_eq!("b2", fs::read_to_string("test/.config/app_b/b2.txt").unwrap());
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
	assert_eq!(
		config,
		Config::try_from(config.to_string().as_str()).unwrap()
	);

	cleanup();
	write_file("test/.config/app/ignore/global_ignore", "global_ignore");
	write_file("test/.config/app/ignore/app_ignore", "app_ignore");
	write_file("test/.config/app/global_ignore", "global_ignore");
	write_file("test/.config/app/app_ignore", "app_ignore");
	config.backup().unwrap();
	assert!(!Path::new("test/backup/.config/app/ignore/global_ignore").is_file());
	assert!(!Path::new("test/backup/.config/app/ignore/app_ignore").is_file());
	assert!(Path::new("test/backup/.config/app/ignore").is_dir());
	assert_eq!(fs::read_to_string("test/backup/.config/app/global_ignore").unwrap(), "global_ignore");
	assert_eq!(fs::read_to_string("test/backup/.config/app/app_ignore").unwrap(), "app_ignore");

	fs::remove_dir_all("test/.config").unwrap();
	write_file("test/backup/.config/app/ignore/global_ignore", "global_ignore");
	write_file("test/backup/.config/app/ignore/app_ignore", "app_ignore");
	config.setup().unwrap();
	assert!(!Path::new("test/.config/app/ignore/global_ignore").is_file());
	assert!(!Path::new("test/.config/app/ignore/app_ignore").is_file());
	assert!(Path::new("test/.config/app/ignore").is_dir());
	assert_eq!(fs::read_to_string("test/.config/app/global_ignore").unwrap(), "global_ignore");
	assert_eq!(fs::read_to_string("test/.config/app/app_ignore").unwrap(), "app_ignore");
}
