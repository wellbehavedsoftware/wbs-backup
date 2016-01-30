use std::env;
//use std::error::Error;
//use std::fmt;
//use std::io;
//use std::io::Read;
//use std::io::Write;
//use std::mem;
use std::process;

use misc::*;

#[ macro_use ]
mod misc;

mod tar;
mod tarpack;
mod wbspack;

fn work () -> Result <(), TfError> {

	let mut stdin =
		std::io::stdin ();

	let mut stdout =
		std::io::stdout ();

	let mut offset = 0;

	let mut headers =
		Vec::new ();

	let mut block_references =
		Vec::new ();

	try! {
		wbspack::write_header (
			&mut stdout,
			&mut offset)
	};

	try! (
		tarpack::write_contents (
			&mut stdin,
			&mut stdout,
			&mut offset,
			&mut headers,
			&mut block_references));

	try! (
		tarpack::write_headers (
			&    headers,
			&mut stdout,
			&mut offset,
			&mut block_references));

	try! (
		wbspack::write_footer (
			&mut stdout,
			&    block_references,
			&mut offset));

	Ok (())

}

fn main () {

	let arguments: Vec <String> =
		env::args ().skip (1).collect ();

	if arguments.len () == 0 {

		stderr! (
			"Usage error");

		process::exit (1);

	}

	if arguments [0] == "create" {

		match work () {

			Ok (()) => {

				stderr! (
					"All done!");

				process::exit (0)

			},

			Err (error) => {

				stderr! (
					"Error: {}",
					error);

				process::exit (1)

			},

		}

	} else {

		stderr! (
			"Unknown command: {}",
			arguments [0]);

		process::exit (1)

	}

}
