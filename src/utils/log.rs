#[macro_export]
macro_rules! error {
	() => {
		eprint!("\n")
	};
	($($arg:tt)*) => {{
		eprint!("{}", $crate::consts::colors::RED);
		eprintln!($($arg)*);
		eprint!("{}", $crate::consts::colors::RESET);
	}};
}

#[macro_export]
macro_rules! warn {
	() => {
		eprint!("\n")
	};
	($($arg:tt)*) => {{
		eprint!("{}", $crate::consts::colors::YELLOW);
		eprintln!($($arg)*);
		eprint!("{}", $crate::consts::colors::RESET);
	}};
}

#[macro_export]
macro_rules! info {
	() => {
		eprint!("\n")
	};
	($($arg:tt)*) => {{
		eprint!("{}", $crate::consts::colors::GREEN);
		eprintln!($($arg)*);
		eprint!("{}", $crate::consts::colors::RESET);
	}};
}

#[macro_export]
macro_rules! log {
	($verbose:expr) => {
		if $verbose {
			eprint!("\n")
		}
	};
	($verbose:expr, $($arg:tt)*) => {{
		if $verbose {
			eprint!("{}", $crate::consts::colors::CYAN);
			eprintln!($($arg)*);
			eprint!("{}", $crate::consts::colors::RESET);
		}
	}};
}
