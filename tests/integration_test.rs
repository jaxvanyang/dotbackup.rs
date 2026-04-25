use dotbackup::cli::Config;

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
fn test_basic() {
	let config_str = include_str!("configs/basic.yml");
	let config = Config::try_from(config_str).unwrap();
	assert_eq!(
		config,
		Config::try_from(config.to_string().as_str()).unwrap()
	);
	config.backup().unwrap();
	config.setup().unwrap();
}

#[test]
fn test_complex_script() {
	let config = Config::try_from(include_str!("configs/complex_script.yml")).unwrap();
	assert!(config.backup().is_ok());
}

#[test]
fn test_ignore() {
	let _config = Config::try_from(include_str!("configs/ignore.yml")).unwrap();
}
