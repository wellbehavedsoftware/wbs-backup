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

fn zunpack (
	backup_path: &str,
) -> Result <(), TfError> {

	let backup_split: Vec <& str> =
		backup_path.splitn (
			2,
			"/backups/",
		).collect ();

	let repository_path =
		& backup_split [0];

	let backup_name =
		& backup_split [1];

	let mut zbackup =
		try! (
			ZBackup::open (
				repository_path));

	let mut input =
		try! (
			zbackup.open_backup (
				backup_name));

	let mut stdout =
		io::stdout ();

	try! (
		wbspack::unpack (
			&mut input,
			&mut stdout));

	Ok (())

}

fn restore (
	backup_path: & str,
) -> Result <(), TfError> {

	let backup_split: Vec <& str> =
		backup_path.splitn (
			2,
			"/backups/",
		).collect ();

	let repository_path =
		& backup_split [0];

	let backup_name =
		& backup_split [1];

	let mut zbackup =
		try! (
			ZBackup::open (
				repository_path));

	try! (
		zbackup.restore (
			backup_name,
			& mut io::stdout ()));

	Ok (())

}

fn restore_test (
	backup_path: & str,
) -> Result <(), TfError> {

	let backup_split: Vec <& str> =
		backup_path.splitn (
			2,
			"/backups/",
		).collect ();

	let repository_path =
		& backup_split [0];

	let backup_name =
		& backup_split [1];

	let mut zbackup =
		try! (
			ZBackup::open (
				repository_path));

	try! (
		zbackup.restore_test (
			backup_name,
			& mut io::stdout ()));

	Ok (())

}

fn main () {

	let arguments: Vec <String> =
		env::args ().skip (1).collect ();

	if arguments.len () == 0 {

		stderrln! (
			"Usage error");

		process::exit (1);

	}

	if arguments [0] == "pack" {

		if arguments.len () != 1 {

			stderrln! (
				"Usage error");

		}

		match pack () {

			Ok (()) => {

				stderrln! (
					"All done!");

				process::exit (0)

			},

			Err (error) => {

				stderrln! (
					"Error: {}",
					error);

				process::exit (1)

			},

		}

	} else if arguments [0] == "unpack" {

		if arguments.len () != 2 {

			stderrln! (
				"Usage error");

		}

		match unpack (& arguments [1]) {

			Ok (()) => {

				stderrln! (
					"All done!");

				process::exit (0)

			},

			Err (error) => {

				stderrln! (
					"Error: {}",
					error);

				process::exit (1)

			},

		}

	} else if arguments [0] == "zunpack" {

		if arguments.len () != 2 {

			stderrln! (
				"Usage error");

		}

		match zunpack (& arguments [1]) {

			Ok (()) => {

				stderrln! (
					"All done!");

				process::exit (0)

			},

			Err (error) => {

				stderrln! (
					"Error: {}",
					error);

				process::exit (1)

			},

		}

	} else if arguments [0] == "restore" {

		if arguments.len () != 2 {

			stderrln! (
				"Usage error");

		}

		match restore (
			& arguments [1],
		) {

			Ok (()) => {

				stderrln! (
					"All done!");

				process::exit (0)

			},

			Err (error) => {

				stderrln! (
					"Error: {}",
					error);

				process::exit (1)

			},

		}

	} else if arguments [0] == "restore-test" {

		if arguments.len () != 2 {

			stderrln! (
				"Usage error");

		}

		match restore_test (
			& arguments [1],
		) {

			Ok (()) => {

				stderrln! (
					"All done!");

				process::exit (0)

			},

			Err (error) => {

				stderrln! (
					"Error: {}",
					error);

				process::exit (1)

			},

		}

	} else {

		stderrln! (
			"Unknown command: {}",
			arguments [0]);

		process::exit (1)

	}

}
