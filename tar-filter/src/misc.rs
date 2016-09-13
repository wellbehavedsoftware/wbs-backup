use std::error::Error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

use protobuf;

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
