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
) -> Result <(), TfError> {

	for header_line in [
		b"WBS PACK 0\0\0\0\0\0\0",
		b"GNU TAR\0\0\0\0\0\0\0\0\0",
	].iter () {

		try! (
			output.write (
				* header_line));

	}

	Ok (())

}

pub fn write_footer (
	output: &mut Write,
	block_references: & Vec <BlockReference>,
) -> Result <(), TfError> {

	try! (
		output.write (
			b"BLOCKS START\0\0\0\0"));

	for block_reference in block_references {

		let binary_block_reference: & [u8; 16] = unsafe {
			mem::transmute::<& BlockReference, & [u8; 16]> (
				block_reference)
		};

		try! (
			output.write (
				binary_block_reference));

	}

	let binary_block_reference_count: & [u8; 8] = unsafe {
		mem::transmute::<& usize, & [u8; 8]> (
			& block_references.len ())
	};

	try! (
		output.write (
			b"BLOCKS END\0\0\0\0\0\0"));

	try! (
		output.write (
			binary_block_reference_count));

	try! (
		output.write (
			b"\0\0\0\0\0\0\0\0"));

	try! (
		output.write (
			b"WBS PACK END\0\0\0\0"));

	Ok (())

}
