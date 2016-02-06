use std::io::Read;
use std::io::Write;

use misc::*;

use tar;
use wbspack;

pub fn pack (
	input: & mut Read,
	packer: & mut wbspack::Packer,
) -> Result <(), TfError> {

	let mut null_count: u64 =
		0;

	let mut deferred: Vec <wbspack::Deferred> =
		vec! ();

	loop {

		let header_bytes =
			match try! (
				read_header (
					input)) {

			Some (bytes) => bytes,

			None => break,

		};

		// read header

		if header_bytes [0 .. 512].as_ref () == [0; 512].as_ref () {

			null_count += 1;

			continue;

		}

		if null_count > 0 {

			panic! (
				"NUL in middle of archive");

		}

		// interpret header

		let header = try! (
			tar::Header::read (
				& header_bytes)
		);

		let blocks = 0
			+ (header.size >> 9)
			+ (if (header.size & 0x1ff) != 0 { 1 } else { 0 });

		match header.typeflag {

			  tar::Type::Regular
			| tar::Type::Link
			| tar::Type::SymbolicLink
			| tar::Type::CharacterSpecial
			| tar::Type::BlockSpecial
			| tar::Type::Directory
			| tar::Type::Fifo => {

				// defer header

				deferred.push (
					try! (
						packer.defer (
							header_bytes)));

				// copy file

				try! (
					packer.align ());

				let mut content_bytes: Vec <u8> =
					vec! [0; 512 * blocks as usize];

				try! (
					input.read_exact (
						& mut content_bytes));

				try! (
					packer.write (
						& content_bytes));

			},

			  tar::Type::LongName
			| tar::Type::LongLink => {

				// defer header

				deferred.push (
					try! (
						packer.defer (
							header_bytes)));

				// defer content

				let mut content_bytes: Vec <u8> =
					Vec::with_capacity (
						blocks as usize * 512);

				unsafe {
					content_bytes.set_len (
						blocks as usize * 512);
				}

				try! (
					input.read_exact (
						& mut content_bytes));

				deferred.push (
					try! (
						packer.defer (
							content_bytes)));

			},

		}

	}

	// write deferred

	try! (
		packer.align ());

	for one_deferred in deferred.iter () {

		try! (
			packer.write_deferred (
				one_deferred));

	}

	// write nulls

	if null_count < 2 {
		panic! ();
	}

	let null_bytes: [u8; 512] =
		[0; 512];

	for _null_count in 0 .. null_count {

		try! (
			packer.write (
				& null_bytes));

	}

	Ok (())

}

fn read_header (
	input: & mut Read,
) -> Result <Option <Vec <u8>>, TfError> {

	let mut header_bytes =
		Vec::with_capacity (
			512);

	unsafe {
		header_bytes.set_len (
			512);
	}

	match try! (
		input.read (
			& mut header_bytes)) {

		0 => Ok (None),
		512 => Ok (Some (header_bytes)),

		bytes_read => panic! (
			"Read {} bytes, expected EOF or 512",
			bytes_read),

	}

}
