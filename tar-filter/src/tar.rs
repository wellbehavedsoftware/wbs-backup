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
	pub blocks: u64,
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
		header_bytes: & [u8],
	) -> Result <Header, TfError> {

		if header_bytes.len () != 512 {
			panic! ();
		}

		let binary_header =
			unsafe {
				mem::transmute::<& u8, & BinaryHeader> (
					& header_bytes [0])
			};

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

			let size =
				try! (
					tar_number_u64 (
						& binary_header.size));

			Ok (Header {

				name: tar_string (
					& binary_header.name),

				mode: try! (
					tar_number_u32 (
						& binary_header.mode)),

				uid: try! (
					tar_number_u32 (
						& binary_header.uid)),

				gid: try! (
					tar_number_u32 (
						& binary_header.gid)),

				size:
					size,

				blocks: 0
					+ (size >> 9)
					+ (if (size & 0x1ff) != 0 { 1 } else { 0 }),

				mtime: try! (
					tar_number_u64 (
						& binary_header.mtime)),

				cksum: try! (
					tar_number_u32 (
						& binary_header.cksum)),

				typeflag: tar_type (
					& binary_header.typeflag),

				linkname: tar_string (
					& binary_header.linkname),

				uname: tar_string (
					& binary_header.uname),

				gname: tar_string (
					& binary_header.gname),

				dev_major: try! (
					tar_number_u32 (
						& binary_header.dev_major)),

				dev_minor: try! (
					tar_number_u32 (
						& binary_header.dev_minor)),

				atime: try! (
					tar_number_u64 (
						& binary_header.atime)),

				ctime: try! (
					tar_number_u64 (
						& binary_header.ctime)),

				offset: try! (
					tar_number_u64 (
						& binary_header.offset)),

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
) -> Result <u64, TfError> {

	if slice [0] == 0 {

		Ok (0)

	} else if slice [0] == 0x80 {

		Ok (u64::from_be (
			unsafe {
				* mem::transmute::<& u8, & u64> (
					& slice [4])
			}
		))

	} else if slice [0] == 0xff {

		panic! ()

	} else {

		let string =
			try! (
				String::from_utf8 (
					tar_string (slice)));

		let number =
			try! (
				u64::from_str_radix (
					& string,
					8));

		Ok (number)

	}

}

fn tar_number_u32 (
	slice: & [u8],
) -> Result <u32, TfError> {

	if slice [0] == 0 {

		Ok (0)

	} else {

		let string =
			try! (
				String::from_utf8 (
					tar_string (slice)));

		let number =
			try! (
				u32::from_str_radix (
					& string,
					8));

		Ok (number)

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
