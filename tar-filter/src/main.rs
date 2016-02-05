extern crate libc;
extern crate protobuf;
extern crate rustc_serialize;

use std::env;
use std::fs::File;
use std::io;
use std::process;

use misc::*;
use zbackup::ZBackup;

#[ macro_use ]
mod misc;

mod lzma;
mod tar;
mod tarpack;
mod wbspack;
mod zbackup;
mod zbackup_proto;

fn pack () -> Result <(), TfError> {

	let mut stdin =
		io::stdin ();

	let mut stdout =
		io::stdout ();

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
		io::stdout ();

	try! (
		wbspack::unpack (
			&mut input,
			&mut stdout));

	Ok (())

}

fn test (
	repository: & str,
	backup_name: & str,
) -> Result <(), TfError> {

	let mut zbackup =
		try! (
			ZBackup::open (
				repository));

	try! (
		zbackup.restore (
			backup_name,
			& mut io::stdout ()));

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

	} else if arguments [0] == "restore" {

		if arguments.len () != 3 {

			stderr! (
				"Usage error");

		}

		match test (
			& arguments [1],
			& arguments [2],
		) {

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
