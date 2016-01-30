use std::env;
use std::fs::File;
use std::process;

use misc::*;

#[ macro_use ]
mod misc;

mod tar;
mod tarpack;
mod wbspack;

fn pack () -> Result <(), TfError> {

	let mut stdin =
		std::io::stdin ();

	let mut stdout =
		std::io::stdout ();

	let mut offset = 0;

	try! {
		wbspack::write_header (
			&mut stdout,
			&mut offset)
	};

	let headers_and_content_blocks =
		try! (
			tarpack::write_contents (
				&mut stdin,
				&mut stdout,
				&mut offset));

	let all_blocks =
		try! (
			tarpack::write_headers (
				&mut stdout,
				&mut offset,
				&    headers_and_content_blocks));

	try! (
		wbspack::write_footer (
			&mut stdout,
			&mut offset,
			&    all_blocks));

	Ok (())

}

fn unpack (
	filename: &str,
) -> Result <(), TfError> {

	let mut input =
		try! (
			File::open (
				filename));

	let mut stdout =
		std::io::stdout ();

	try! (
		wbspack::unpack (
			&mut input,
			&mut stdout));

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

	if arguments [0] == "pack" {

		if arguments.len () != 1 {

			stderr! (
				"Usage error");

		}

		match pack () {

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

	} else if arguments [0] == "unpack" {

		if arguments.len () != 2 {

			stderr! (
				"Usage error");

		}

		match unpack (& arguments [1]) {

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
