use std::error::Error;
use std::fmt;
use std::io;

macro_rules! stderr {

	( $ ( $arg : tt ) * ) => (

		use std::io::Write;

		match writeln! (
			&mut ::std::io::stderr (),
			$ ( $arg ) *,
		) {

			Ok (_) => {},

			Err (error) => panic! (
				"Unable to write to stderr: {}",
				error),

		}

	)

}

#[ derive (Debug) ]
pub struct TfError {

	pub error_message: String,

}

impl fmt::Display for TfError {

	fn fmt (
		&self,
		formatter: &mut fmt::Formatter,
	) -> fmt::Result {

		write! (
			formatter,
			"{}",
			self.error_message)

	}

}

impl From <String> for TfError {

	fn from (error: String) -> TfError {
		TfError {
			error_message: error,
		}
	}

}

impl From <io::Error> for TfError {

	fn from (error: io::Error) -> TfError {
		TfError {
			error_message: error.description ().to_string (),
		}
	}

}
