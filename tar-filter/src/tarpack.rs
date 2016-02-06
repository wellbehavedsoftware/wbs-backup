use std::io::Read;
use std::io::Write;

use misc::*;

use tar;
use wbspack;

struct TarPacker <'a, 'b: 'a> {
	input: & 'a mut Read,
	packer: & 'a mut wbspack::Packer <'b>,
	deferred: Vec <wbspack::Deferred>,
	null_count: u64,
}

pub fn pack (
	input: & mut Read,
	packer: & mut wbspack::Packer,
) -> Result <(), TfError> {

	let mut tar_packer =
		TarPacker {
			input: input,
			packer: packer,
			deferred: vec! (),
			null_count: 0,
		};

	while try! (
		tar_packer.process_one_entry ()) {

	}

	try! (
		tar_packer.write_deferred ());

	try! (
		tar_packer.write_nulls ());

	Ok (())

}

impl <'a, 'b> TarPacker <'a, 'b> {

	pub fn process_one_entry (
		& mut self,
	) -> Result <bool, TfError> {

		let header_bytes =
			match try! (
				self.read_header ()) {

			Some (bytes) => bytes,

			None => return Ok (false),

		};

		// read header

		if header_bytes [0 .. 512].as_ref () == [0; 512].as_ref () {

			self.null_count += 1;

			return Ok (true);

		}

		if self.null_count > 0 {

			panic! (
				"NUL in middle of archive");

		}

		// interpret header

		try! (
			self.interpret_header (
				header_bytes));

		Ok (true)

	}

	fn read_header (
		& mut self
	) -> Result <Option <Vec <u8>>, TfError> {

		let mut header_bytes =
			Vec::with_capacity (
				512);

		unsafe {
			header_bytes.set_len (
				512);
		}

		match try! (
			self.input.read (
				& mut header_bytes)) {

			0 => Ok (None),
			512 => Ok (Some (header_bytes)),

			bytes_read => panic! (
				"Read {} bytes, expected EOF or 512",
				bytes_read),

		}

	}

	fn interpret_header (
		& mut self,
		header_bytes: Vec <u8>,
	) -> Result <(), TfError> {

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

				self.deferred.push (
					try! (
						self.packer.defer (
							header_bytes)));

				// copy file

				try! (
					self.packer.align ());

				let mut content_bytes: Vec <u8> =
					vec! [0; 512 * blocks as usize];

				try! (
					self.input.read_exact (
						& mut content_bytes));

				try! (
					self.packer.write (
						& content_bytes));

			},

			  tar::Type::LongName
			| tar::Type::LongLink => {

				// defer header

				self.deferred.push (
					try! (
						self.packer.defer (
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
					self.input.read_exact (
						& mut content_bytes));

				self.deferred.push (
					try! (
						self.packer.defer (
							content_bytes)));

			},

		}

		Ok (())

	}

	fn write_deferred (
		& mut self,
	) -> Result <(), TfError> {

		try! (
			self.packer.align ());

		for one_deferred in self.deferred.iter () {

			try! (
				self.packer.write_deferred (
					one_deferred));

		}

		Ok (())

	}

	fn write_nulls (
		& mut self,
	) -> Result <(), TfError> {

		if self.null_count < 2 {
			panic! ();
		}

		let null_bytes: [u8; 512] =
			[0; 512];

		for _null_count in 0 .. self.null_count {

			try! (
				self.packer.write (
					& null_bytes));

		}

		Ok (())

	}

}
