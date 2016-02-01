use std::io::Read;
use std::io::Write;

use misc::*;

use tar;
use wbspack;

pub struct Work {
	pub blocks: Vec <WorkBlock>,
	pub null_count: u64,
}

pub enum WorkBlock {

	Content {
		header: [u8; 512],
		reference: wbspack::BlockReference,
	},

	Metadata {
		header: [u8; 512],
		content: Vec <u8>,
	},

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

				// store header and content block

				work.blocks.push (
					WorkBlock::Content {
						header: header_bytes,
						reference: wbspack::BlockReference {
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

			},

			tar::Type::LongName
			| tar::Type::LongLink => {

				// read content

				let mut content_bytes: Vec <u8> =
					Vec::with_capacity (
						blocks as usize * 512);

				unsafe {
					content_bytes.set_len (
						blocks as usize * 512);
				}

				try! (
					input.read_exact (
						&mut content_bytes));

				// store header and content

				work.blocks.push (
					WorkBlock::Metadata {
						header: header_bytes,
						content: content_bytes,
					}
				);

			},

		}

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

		match block {

			& WorkBlock::Content {
				header,
				reference,
			} => {

				try! (
					output.write (
						& header));

				all_blocks.push (
					wbspack::BlockReference {
						offset: * offset,
						size: 512,
					}
				);

				all_blocks.push (
					reference);

				* offset += 512;

			},

			& WorkBlock::Metadata {
				header,
				ref content,
			} => {

				try! (
					output.write (
						& header));

				all_blocks.push (
					wbspack::BlockReference {
						offset: * offset,
						size: 512,
					}
				);

				* offset += 512;

				try! (
					output.write (
						& content));

				all_blocks.push (
					wbspack::BlockReference {
						offset: * offset,
						size: content.len () as u64,
					}
				);

				* offset +=
					content.len () as u64;

			}

		}

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
