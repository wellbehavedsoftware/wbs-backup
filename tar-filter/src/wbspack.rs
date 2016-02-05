use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::mem;

use misc::*;

#[ repr (C) ]
#[ derive (Copy, Clone) ]
pub struct BlockReference {

	pub offset: u64,
	pub size: u64,

}

pub fn write_header (
	output: & mut Write,
	offset: & mut u64,
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

pub fn read_header (
	input: & mut Read,
) -> Result <(), TfError> {

	let mut header_line: [u8; 16] =
		[0; 16];

	// read magic header

	try! (
		input.read_exact (
			& mut header_line));

	if header_line != * b"WBS PACK\0\0\0\0\0\0\0\0" {

		return Err (TfError {
			error_message: String::from (
				"Not a WBS pack file"),
		});

	}

	// read header start

	try! (
		input.read_exact (
			& mut header_line));

	if header_line != * b"HEADER START\0\0\0\0" {

		return Err (TfError {
			error_message: String::from (
				"Not a WBS pack file"),
		});

	}

	// read rest of header

	let mut got_version_0 =
		false;

	loop {

		try! (
			input.read_exact (
				& mut header_line));

		// read version 0

		if header_line == * b"VERSION 0\0\0\0\0\0\0\0" {
			got_version_0 = true;
		}

		// read header end

		if header_line == * b"HEADER END\0\0\0\0\0\0" {
			break;
		}

	}

	// check version

	if ! got_version_0 {

		return Err (TfError {
			error_message: String::from (
				"Unknown WBS pack file version"),
		});

	}

	Ok (())

}

pub fn write_footer (
	output: & mut Write,
	offset: & mut u64,
	block_references: & Vec <BlockReference>,
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

	* offset += 8;

	// write blocks length

	let blocks_length: u64 =
		block_references.len () as u64;

	let binary_blocks_length: & [u8; 8] = unsafe {
		mem::transmute::<& u64, & [u8; 8]> (
			& blocks_length)
	};

	try! (
		output.write (
			binary_blocks_length));

	* offset += 8;

	// write end

	try! (
		output.write (
			b"WBS PACK END\0\0\0\0"));

	Ok (())

}

pub fn read_footer <F: Read + Seek> (
	input: & mut F,
) -> Result <Vec <BlockReference>, TfError> {

	let mut footer_line: [u8; 16] =
		[0; 16];

	// read magic footer

	try! (
		input.seek (
			SeekFrom::End (-16)));

	try! (
		input.read_exact (
			& mut footer_line));

	if footer_line != * b"WBS PACK END\0\0\0\0" {

		return Err (TfError {
			error_message: String::from (
				"Not a WBS pack file"),
		});

	}

	// read blocks location

	try! (
		input.seek (
			SeekFrom::End (-32)));

	try! (
		input.read_exact (
			& mut footer_line));

	let blocks_location = unsafe {
		mem::transmute::<[u8; 16], BlockReference> (
			footer_line)
	};

	// read blocks start

	try! (
		input.seek (
			SeekFrom::Start (
				blocks_location.offset)));

	try! (
		input.read_exact (
			& mut footer_line));

	if footer_line != * b"BLOCKS START\0\0\0\0" {

		return Err (TfError {
			error_message: String::from (
				"Not a WBS pack file"),
		});

	}

	// read blocks

	let mut block_references: Vec <BlockReference> =
		Vec::with_capacity (
			blocks_location.size as usize);

	for _block_index in 0 .. blocks_location.size {

		try! (
			input.read_exact (
				& mut footer_line));

		block_references.push (unsafe {
			mem::transmute::<[u8; 16], BlockReference> (
				footer_line)
		});

	}

	// read blocks end

	try! (
		input.read_exact (
			& mut footer_line));

	if footer_line != * b"BLOCKS END\0\0\0\0\0\0" {

		return Err (TfError {
			error_message: String::from (
				"Not a WBS pack file"),
		});

	}

	Ok (block_references)

}

pub fn copy_blocks <Input: Read + Seek> (
	input: & mut Input,
	output: & mut Write,
	blocks: & Vec <BlockReference>,
) -> Result <(), TfError> {

	for block in blocks {

		try! (
			input.seek (
				SeekFrom::Start (
					block.offset)));

		let mut block_reader =
			input.take (
				block.size);

		try! (
			io::copy (
				& mut block_reader,
				output));

	}

	Ok (())

}

pub fn unpack <F: Read + Seek> (
	input: & mut F,
	output: & mut Write,
) -> Result <(), TfError> {

	try! (
		read_header (
			input));

	let block_references =
		try! (
			read_footer (
				input));

	try! (
		copy_blocks (
			input,
			output,
			& block_references));

	Ok (())

}
