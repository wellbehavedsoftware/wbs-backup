use std::error::Error;
use std::io;

pub fn io_result <Type> (
	result: Result <Type, io::Error>,
) -> Result <Type, String> {

	result.map_err (
		|io_error|
		io_error.description ().to_string ()
	)

}

// ex: noet ts=4 filetype=rust
