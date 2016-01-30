use std::io::Read;
use std::io::Write;

use misc::*;

use tar;
use wbspack;

pub struct Work {
	pub blocks: Vec <WorkBlock>,
	pub null_count: u64,
}

pub struct WorkBlock {
	pub header: [u8; 512],
	pub content_block: wbspack::BlockReference,
}

pub fn write_contents (
	input: &mut Read,
	output: &mut Write,
	offset: &mut u64,
) -> Result <Work, TfError> {

	let mut header_bytes: [u8; 512] =
		[0; 512];

	let mut work = Work {
		blocks: vec! {},
		null_count: 0,
	};

	loop {

		// read header

		if input.read_exact (
			&mut header_bytes,
		).is_err () {

			if work.null_count >= 2 {
				break;
			}

		}

		if header_bytes.as_ref () == [0; 512].as_ref () {

			work.null_count += 1;

			continue;

		}

		if work.null_count > 0 {

			panic! (
				"NUL in middle of archive");

		}

		// interpret header

		let header = try! (
			tar::Header::read (
				& header_bytes)
		);

		// store header and content block

		let blocks = 0
			+ (header.size >> 9)
			+ (if (header.size & 0x1ff) != 0 { 1 } else { 0 });

		work.blocks.push (
			WorkBlock {
				header: header_bytes,
				content_block: wbspack::BlockReference {
					offset: * offset,
					size: blocks * 512,
				}
			}
		);

		// copy file

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

		* offset +=
			blocks * 512;

	}

	Ok (work)

}

pub fn write_headers (
	output: &mut Write,
	offset: &mut u64,
	work: & Work,
) -> Result <Vec <wbspack::BlockReference>, TfError> {

	let mut all_blocks: Vec <wbspack::BlockReference> =
		Vec::with_capacity (
			work.blocks.len () * 2);

	// write headers

	for block in work.blocks.iter () {

		try! (
			output.write (
				& block.header));

		all_blocks.push (
			wbspack::BlockReference {
				offset: * offset,
				size: 512,
			}
		);

		all_blocks.push (
			block.content_block);

		* offset += 512;

	}

	// write nulls

	let null_bytes: [u8; 512] =
		[0; 512];

	for _null_count in 0 .. work.null_count {

		try! (
			output.write (
				& null_bytes));

	}

	all_blocks.push (
		wbspack::BlockReference {
			offset: * offset,
			size: work.null_count * 512,
		}
	);

	* offset += work.null_count * 512;

	// return

	Ok (all_blocks)

}
