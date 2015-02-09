#![macro_use]

extern crate time;

#[macro_export]
macro_rules! log {

	($($arg:tt)*) => {

		println! (
			"{}: {}",
			time_format_pretty (time::get_time ()),
			format! ($($arg)*))

	}

}
