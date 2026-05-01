#[macro_export]
macro_rules! error {
	() => {
		eprintln!();
	};
	($($arg:tt)*) => {{
		eprint!("{}==>{} ", $crate::consts::colors::RED, $crate::consts::colors::RESET);
		eprintln!($($arg)*);
	}};
}

#[macro_export]
macro_rules! warn {
	() => {
		eprintln!();
	};
	($($arg:tt)*) => {{
		eprint!("{}==>{} ", $crate::consts::colors::YELLOW, $crate::consts::colors::RESET);
		eprintln!($($arg)*);
	}};
}

#[macro_export]
macro_rules! info {
	() => {
		eprintln!();
	};
	($($arg:tt)*) => {{
		eprint!("{}==>{} ", $crate::consts::colors::GREEN, $crate::consts::colors::RESET);
		eprintln!($($arg)*);
	}};
}

#[macro_export]
macro_rules! log {
	($verbose:expr) => {
		if $verbose {
			eprintln!();
		}
	};
	($verbose:expr, $($arg:tt)*) => {{
		if $verbose {
		eprint!("{}==>{} ", $crate::consts::colors::CYAN, $crate::consts::colors::RESET);
		eprintln!($($arg)*);
		}
	}};
}
