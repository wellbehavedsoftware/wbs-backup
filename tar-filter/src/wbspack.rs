use std::io::Write;
use std::mem;

use misc::*;

#[ repr (C) ]
pub struct BlockReference {

	pub offset: u64,
	pub size: u64,

}

pub fn write_header (
	output: &mut Write,
	offset: &mut u64,
) -> Result <(), TfError> {

	for header_line in [
		b"WBS PACK\0\0\0\0\0\0\0\0",
		b"HEADER START\0\0\0\0",
		b"VERSION 0\0\0\0\0\0\0\0",
		b"TARPACK 0\0\0\0\0\0\0\0",
		b"HEADER END\0\0\0\0\0\0",
	].iter () {

		try! (
			output.write (
				* header_line));

		* offset += 16;

	}

	Ok (())

}

pub fn write_footer (
	output: &mut Write,
	block_references: & Vec <BlockReference>,
	offset: &mut u64,
) -> Result <(), TfError> {

	let blocks_offset =
		* offset;

	// write blocks

	try! (
		output.write (
			b"BLOCKS START\0\0\0\0"));

	* offset += 16;

	for block_reference in block_references {

		let binary_block_reference: & [u8; 16] = unsafe {
			mem::transmute::<& BlockReference, & [u8; 16]> (
				block_reference)
		};

		try! (
			output.write (
				binary_block_reference));

		* offset += 16;

	}

	try! (
		output.write (
			b"BLOCKS END\0\0\0\0\0\0"));

	* offset += 16;

	// write blocks offset

	let binary_blocks_offset: & [u8; 8] = unsafe {
		mem::transmute::<& u64, & [u8; 8]> (
			& blocks_offset)
	};

	try! (
		output.write (
			binary_blocks_offset));

	try! (
		output.write (
			b"\0\0\0\0\0\0\0\0"));

	* offset += 16;

	// write end

	try! (
		output.write (
			b"WBS PACK END\0\0\0\0"));

	Ok (())

}
