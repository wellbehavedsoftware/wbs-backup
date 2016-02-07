use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

use protobuf;

macro_rules! stderr {

	( $ ( $arg : tt ) * ) => (

		match write! (
			&mut ::std::io::stderr () as &mut ::std::io::Write,
			$ ( $arg ) *,
		) {

			Ok (_) => {},

			Err (error) => panic! (
				"Unable to write to stderr: {}",
				error),

		}

	)

}

macro_rules! stderrln {

	( $ ( $arg : tt ) * ) => (

		match writeln! (
			&mut ::std::io::stderr () as &mut ::std::io::Write,
			$ ( $arg ) *,
		) {

			Ok (_) => {},

			Err (error) => panic! (
				"Unable to write to stderr: {}",
				error),

		}

	)

}

pub fn to_array (
	slice: & [u8],
) -> [u8; 24] {

	[
		slice [0],  slice [1],  slice [2],  slice [3],  slice [4],  slice [5],
		slice [6],  slice [7],  slice [8],  slice [9],  slice [10], slice [11],
		slice [12], slice [13], slice [14], slice [15], slice [16], slice [17],
		slice [18], slice [19], slice [20], slice [21], slice [22], slice [23],
	]

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

impl From <protobuf::error::ProtobufError> for TfError {

	fn from (error: protobuf::error::ProtobufError) -> TfError {
		TfError {
			error_message: error.description ().to_string (),
		}
	}

}

impl From <Box <Error>> for TfError {

	fn from (error: Box <Error>) -> TfError {
		TfError {
			error_message: error.description ().to_string (),
		}
	}

}

impl From <FromUtf8Error> for TfError {

	fn from (error: FromUtf8Error) -> TfError {
		TfError {
			error_message: error.description ().to_string (),
		}
	}

}

impl From <ParseIntError> for TfError {

	fn from (error: ParseIntError) -> TfError {
		TfError {
			error_message: error.description ().to_string (),
		}
	}

}
