use std::fmt::Display;

#[derive(Debug, Clone, Default)]
pub enum Type {
	Config,
	Argument,
	System,
	App,
	#[default]
	Unknown,
}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Config => write!(f, "configuration error"),
			Self::Argument => write!(f, "CLI argument error"),
			Self::System => write!(f, "system error"),
			Self::App => write!(f, "application error"),
			Self::Unknown => write!(f, "unknown error"),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Error {
	pub r#type: Type,
	pub msg: String,
}

impl Error {
	#[must_use]
	pub fn new(r#type: Type, msg: String) -> Self {
		Self { r#type, msg }
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}: {}", self.r#type, self.msg)
	}
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! arg_error {
	($($arg:tt)*) => {
		$crate::error::Error::new($crate::error::Type::Argument, format!($($arg)*))
	};
}

#[macro_export]
macro_rules! sys_error {
	($($arg:tt)*) => {
		$crate::error::Error::new($crate::error::Type::System, format!($($arg)*))
	};
}

#[macro_export]
macro_rules! config_error {
	($($arg:tt)*) => {
		$crate::error::Error::new($crate::error::Type::Config, format!($($arg)*))
	};
}
