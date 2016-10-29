#[ macro_use ]
extern crate clap;

use std::error::Error;
use std::hash::Hasher;
use std::hash::SipHasher;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process;

struct Arguments {
	buckets: u64,
	value: u64,
}

fn main () {

	let arguments =
		parse_arguments ();

	do_filter (
		& arguments,
	).unwrap_or_else (
		|io_error| {

		writeln! (
			& mut io::stderr (),
			"IO error: {}",
			io_error.description ()
		).unwrap ();

		process::exit (1);

	});

}

fn parse_arguments (
) -> Arguments {

	let argument_matches = (
		clap::App::new ("Hash Filter")

		.arg (
			clap::Arg::with_name ("buckets")
				.long ("buckets")
				.value_name ("BUCKETS")
				.help ("Total number of buckets to place values in")
				.takes_value (true)
				.required (true)
		)

		.arg (
			clap::Arg::with_name ("value")
				.long ("value")
				.value_name ("VALUE")
				.help ("Bucket value to allow to pass filter")
				.takes_value (true)
				.required (true)
		)

	).get_matches ();

	let buckets = value_t! (
		argument_matches.value_of ("buckets"),
		u64
	).unwrap_or_else (
		|error|
		error.exit ()
	);

	let value = value_t! (
		argument_matches.value_of ("value"),
		u64
	).unwrap_or_else (
		|error|
		error.exit ()
	);

	if buckets < 1 {

		writeln! (
			& mut io::stderr (),
			"Must have at least one bucket"
		).unwrap ();

		process::exit (1);

	}

	if value >= buckets {

		writeln! (
			& mut io::stderr (),
			"Value must be less than number of buckets"
		).unwrap ();

		process::exit (1);

	}

	Arguments {
		buckets: buckets,
		value: value,
	}

}

fn do_filter (
	arguments: & Arguments,
) -> io::Result <()> {

	let input =
		BufReader::new (
			io::stdin ());

	let mut output =
		io::stdout ();

	let mut hasher =
		SipHasher::new ();

	for item_result in input.split (0u8) {

		let item =
			try! (
				item_result);

		hasher.write (
			& item);

		let hash =
			hasher.finish ();

		let bucket =
			hash % arguments.buckets;

		if arguments.value == bucket {

			try! (
				output.write (
					& item));

			try! (
				output.write (
					b"\0"));

		}

	}

	Ok (())

}

// ex: noet ts=4 filetype=rust
