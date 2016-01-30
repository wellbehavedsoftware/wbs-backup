use std::io::Read;
use std::io::Write;

use misc::*;

use tar;
use wbspack;

pub fn write_contents (
	input: &mut Read,
	output: &mut Write,
	offset: &mut u64,
	headers: &mut Vec <[u8; 512]>,
	block_references: &mut Vec <wbspack::BlockReference>,
) -> Result <(), TfError> {

	let mut header_bytes: [u8; 512] =
		[0; 512];

	let mut null_count = 0;

	loop {

		// read header

		if input.read_exact (
			&mut header_bytes,
		).is_err () {

			if null_count >= 2 {
				return Ok (())
			}

		}

		if header_bytes.as_ref () == [0; 512].as_ref () {

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

		// store header

		headers.push (
			header_bytes);

		// copy file

		let blocks = 0
			+ (header.size >> 9)
			+ (if (header.size & 0x1ff) != 0 { 1 } else { 0 });

		let mut content_bytes: [u8; 512] =
			[0; 512];

		for _block_index in 0 .. blocks {

			try! (
				input.read_exact (
					&mut content_bytes));

			try! (
				output.write (
					& content_bytes));

		}

		block_references.push (
			wbspack::BlockReference {
				offset: * offset,
				size: blocks * 512,
			}
		);

		* offset +=
			blocks * 512;

	}

}

pub fn write_headers (
	headers: & Vec <[u8; 512]>,
	output: &mut Write,
	offset: &mut u64,
	block_references: &mut Vec <wbspack::BlockReference>,
) -> Result <(), TfError> {

	for header_bytes in headers {

		try! (
			output.write (
				header_bytes));

		block_references.push (
			wbspack::BlockReference {
				offset: * offset,
				size: 512,
			}
		);

		* offset += 512;

	}

	Ok (())

}
