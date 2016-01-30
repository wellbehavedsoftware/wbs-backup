use std::mem;

use misc::*;

#[ repr (C) ]
struct BinaryHeader {

	name: [u8; 100],
	mode: [u8; 8],
	uid: [u8; 8],
	gid: [u8; 8],
	size: [u8; 12],
	mtime: [u8; 12],
	cksum: [u8; 8],
	typeflag: [u8; 1],
	linkname: [u8; 100],
	magic: [u8; 6],
	version: [u8; 2],
	uname: [u8; 32],
	gname: [u8; 32],
	dev_major: [u8; 8],
	dev_minor: [u8; 8],
	atime: [u8; 12],
	ctime: [u8; 12],
	offset: [u8; 12],
	longnames: [u8; 4],
	unused: [u8; 1],
	sparse: [BinarySparseHeader; 4],
	isextended: [u8; 1],
	realsize: [u8; 12],
	pad: [u8; 17],

}

#[ repr (C) ]
struct BinarySparseHeader {

	offset: [u8; 12],
	numbytes: [u8; 12],

}

pub struct Header {

	pub name: Vec <u8>,
	pub mode: u32,
	pub uid: u32,
	pub gid: u32,
	pub size: u64,
	pub mtime: u64,
	pub cksum: u32,
	pub typeflag: Type,
	pub linkname: Vec <u8>,
	pub uname: Vec <u8>,
	pub gname: Vec <u8>,
	pub dev_major: u32,
	pub dev_minor: u32,
	pub atime: u64,
	pub ctime: u64,
	pub offset: u64,

}

#[ derive (Debug) ]
pub enum Type {

	Regular,
	Link,
	SymbolicLink,
	CharacterSpecial,
	BlockSpecial,
	Directory,
	Fifo,

	LongName,
	LongLink,

}

impl Header {

	pub fn read (
		header_bytes: & [u8; 512],
	) -> Result <Header, TfError> {

		let binary_header: BinaryHeader =
			unsafe { mem::transmute (* header_bytes) };

		if binary_header.magic != * b"ustar " {

			Err (TfError {
				error_message: format! (
					"Unrecognised tar format: {:?} {:?}",
					binary_header.magic,
					binary_header.version),
			})

		} else if binary_header.version != * b" \0" {

			Err (TfError {
				error_message: format! (
					"Unrecognised gnu tar version: {:?}",
					binary_header.version),
			})

	 	} else {

			Ok (Header {

				name: tar_string (
					& binary_header.name),

				mode: tar_number_u32 (
					& binary_header.mode),

				uid: tar_number_u32 (
					& binary_header.uid),

				gid: tar_number_u32 (
					& binary_header.gid),

				size: tar_number_u64 (
					& binary_header.size),

				mtime: tar_number_u64 (
					& binary_header.mtime),

				cksum: tar_number_u32 (
					& binary_header.cksum),

				typeflag: tar_type (
					& binary_header.typeflag),

				linkname: tar_string (
					& binary_header.linkname),

				uname: tar_string (
					& binary_header.uname),

				gname: tar_string (
					& binary_header.gname),

				dev_major: tar_number_u32 (
					& binary_header.dev_major),

				dev_minor: tar_number_u32 (
					& binary_header.dev_minor),

				atime: tar_number_u64 (
					& binary_header.atime),

				ctime: tar_number_u64 (
					& binary_header.ctime),

				offset: tar_number_u64 (
					& binary_header.offset),

/*
				longnames: [u8; 4],
				unused: [u8; 1],
				sparse: [BinarySparseHeader; 4],
				isextended: [u8; 1],
				realsize: [u8; 12],
				pad: [u8; 17],
*/

			})

		}

	}

}

fn tar_string (
	slice: & [u8],
) -> Vec <u8> {

	match slice.iter ().position (
		|index| * index == 0,
	) {

		Some (index) =>
			slice [ .. index ].to_vec (),

		None =>
			slice.to_vec (),

	}

}

fn tar_number_u64 (
	slice: & [u8],
) -> u64 {

	if slice [0] == 0 {
		0
	} else {
		u64::from_str_radix (
			& String::from_utf8 (
				tar_string (slice),
			).unwrap (),
			8,
		).unwrap ()
	}

}

fn tar_number_u32 (
	slice: & [u8],
) -> u32 {

	if slice [0] == 0 {
		0
	} else {
		u32::from_str_radix (
			& String::from_utf8 (
				tar_string (slice),
			).unwrap (),
			8,
		).unwrap ()
	}

}

fn tar_type (
	typeflag: & [u8; 1],
) -> Type {

	match typeflag [0] {

		b'0' => Type::Regular,
		b'1' => Type::Link,
		b'2' => Type::SymbolicLink,
		b'3' => Type::CharacterSpecial,
		b'4' => Type::BlockSpecial,
		b'5' => Type::Directory,
		b'6' => Type::Fifo,

		b'K' => Type::LongLink,
		b'L' => Type::LongName,

		_ => {

			panic! (
				"Unrecognised typeflag: {:?}",
				typeflag [0])

		}

	}

}
