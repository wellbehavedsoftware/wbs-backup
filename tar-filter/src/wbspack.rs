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

pub struct Packer <'a> {

	output: & 'a mut Write,
	offset: u64,
	alignment: u64,

	block_references: Vec <BlockReference>,

}

pub struct Deferred {

	content: Vec <u8>,
	index: u64,

}

impl <'a> Packer <'a> {

	pub fn new (
		output: & 'a mut Write,
		offset: u64,
		alignment: u64,
	) -> Result <Packer <'a>, TfError> {

		Ok (Packer {

			output: output,
			offset: offset,
			alignment: alignment,

			block_references: vec! (),

		})

	}

	pub fn write (
		& mut self,
		data: & [u8],
	) -> Result <(), TfError> {

		try! (
			self.output.write (
				data));

		self.block_references.push (
			BlockReference {
				offset: self.offset,
				size: data.len () as u64,
			});

		self.offset +=
			data.len () as u64;

		Ok (())

	}

	pub fn defer (
		& mut self,
		content: Vec <u8>,
	) -> Result <Deferred, TfError> {

		let deferred =
			Deferred {
				content: content,
				index: self.block_references.len () as u64,
			};

		self.block_references.push (
			BlockReference {
				offset: 0,
				size: 0,
			});

		Ok (deferred)

	}

	pub fn write_deferred (
		& mut self,
		deferred: & Deferred,
	) -> Result <(), TfError> {

		try! (
			self.output.write (
				& deferred.content));

		self.block_references [deferred.index as usize] =
			BlockReference {
				offset: self.offset,
				size: deferred.content.len () as u64,
			};

		self.offset +=
			deferred.content.len () as u64;

		Ok (())

	}

	pub fn align (
		& mut self,
	) -> Result <(), TfError> {

		let remainder =
			self.offset % self.alignment;

		let padding =
			self.alignment - remainder;

		if remainder != 0 {

			let zeroes: Vec <u8> =
				vec! [0; padding as usize];

			try! (
				self.output.write (
					& zeroes));

			self.offset +=
				padding;

		}

		Ok (())

	}

	pub fn write_header (
		& mut self,
	) -> Result <(), TfError> {

		for header_line in [
			b"WBS PACK\0\0\0\0\0\0\0\0",
			b"HEADER START\0\0\0\0",
			b"VERSION 0\0\0\0\0\0\0\0",
			b"TARPACK 0\0\0\0\0\0\0\0",
			b"HEADER END\0\0\0\0\0\0",
		].iter () {

			try! (
				self.output.write (
					* header_line));

			self.offset += 16;

		}

		Ok (())

	}

	pub fn write_footer (
		& mut self,
	) -> Result <(), TfError> {

		try! (
			self.align ());

		let blocks_offset =
			self.offset;

		// write blocks

		try! (
			self.output.write (
				b"BLOCKS START\0\0\0\0"));

		self.offset += 16;

		for block_reference in self.block_references.iter () {

			let binary_block_reference: & [u8; 16] =
				unsafe {
					mem::transmute::<& BlockReference, & [u8; 16]> (
						block_reference)
				};

			try! (
				self.output.write (
					binary_block_reference));

			self.offset +=
				512;

		}

		try! (
			self.output.write (
				b"BLOCKS END\0\0\0\0\0\0"));

		self.offset += 16;

		// write blocks offset

		let binary_blocks_offset: & [u8; 8] = unsafe {
			mem::transmute::<& u64, & [u8; 8]> (
				& blocks_offset)
		};

		try! (
			self.output.write (
				binary_blocks_offset));

		self.offset += 8;

		// write blocks length

		let blocks_length: u64 =
			self.block_references.len () as u64;

		let binary_blocks_length: & [u8; 8] = unsafe {
			mem::transmute::<& u64, & [u8; 8]> (
				& blocks_length)
		};

		try! (
			self.output.write (
				binary_blocks_length));

		self.offset += 8;

		// write end

		try! (
			self.output.write (
				b"WBS PACK END\0\0\0\0"));

		self.offset += 8;

		Ok (())

	}

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
