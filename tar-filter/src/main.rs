use std::io::Read;
use std::io::Write;
use std::mem;

macro_rules! stderr {

	($($arg:tt)*) => (

		match writeln! (
			&mut ::std::io::stderr (),
			$($arg)*,
		) {

			Ok (_) => {},

			Err (x) => panic! (
				"Unable to write to stderr (file handle closed?): {}",
				x),

		}

	)

}

struct BlockReference {
	offset: u64,
	size: u64,
}

#[repr (C)]
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

#[repr (C)]
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

#[derive (Debug)]
pub enum Type {
	Regular,
	Link,
	SymbolicLink,
	CharacterSpecial,
	BlockSpecial,
	Directory,
	Fifo,
}

impl Header {

	pub fn read (
		header_bytes: & [u8; 512],
	) -> Result <Header, String> {

		let binary_header: BinaryHeader =
			unsafe { mem::transmute (* header_bytes) };

		if binary_header.magic != * b"ustar " {

			Err (
				format! (
					"Unrecognised tar format: {:?} {:?}",
					binary_header.magic,
					binary_header.version))

		} else if binary_header.version != * b" \0" {

			Err (
				format! (
					"Unrecognised gnu tar version: {:?}",
					binary_header.version))

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

		_ => {

			panic! (
				"Unrecognised typeflag: {:?}",
				typeflag [0])

		}

	}

}

fn write_tar_contents (
	input: &mut Read,
	output: &mut Write,
	headers: &mut Vec <[u8; 512]>,
	block_references: &mut Vec <BlockReference>,
) -> Result <u64, String> {

	let mut header_bytes: [u8; 512] =
		[0; 512];

	let mut null_count = 0;

	let mut offset = 0;

	loop {

		// read header

		if input.read_exact (
			&mut header_bytes,
		).is_err () {

			if null_count >= 2 {
				return Ok (offset)
			}

		}

		if header_bytes.as_ref () == [0; 512].as_ref () {

			stderr! ("NUL");

			null_count += 1;

			continue;

		}

		if null_count > 0 {

			panic! (
				"NUL in middle of archive");

		}

		// interpret header

		let header = try! (
			Header::read (
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

			try! {
				input.read_exact (
					&mut content_bytes,
				).or (
					Err ("Input error"),
				)
			};

			output.write (
				& content_bytes);

		}

		block_references.push (
			BlockReference {
				offset: offset,
				size: blocks * 512,
			}
		);

		offset +=
			blocks * 512;

	}

}

fn write_tar_headers (
	headers: & Vec <[u8; 512]>,
	output: &mut Write,
	initial_offset: u64,
	block_references: &mut Vec <BlockReference>,
) -> Result <u64, String> {

	let mut offset =
		initial_offset;

	for header_bytes in headers {

		output.write (
			header_bytes);

		block_references.push (
			BlockReference {
				offset: offset,
				size: 512,
			}
		);

		offset += 512;

	}

	Ok (offset)

}

fn write_wbspack_footer (
	output: &mut Write,
	block_references: & Vec <BlockReference>,
) -> Result <(), String> {

	output.write (
		b"BLOCKS START\0\0\0\0");

	for block_reference in block_references {

		let binary_block_reference: & [u8; 16] = unsafe {
			mem::transmute::<& BlockReference, & [u8; 16]> (
				block_reference)
		};

		output.write (
			binary_block_reference);

	}

	let binary_block_reference_count: & [u8; 8] = unsafe {
		mem::transmute::<& usize, & [u8; 8]> (
			& block_references.len ())
	};

	output.write (
		b"BLOCKS END\0\0\0\0\0\0");

	output.write (
		binary_block_reference_count);

	output.write (
		b"\0\0\0\0\0\0\0\0");

	output.write (
		b"WBS PACK END\0\0\0\0");

	Ok (())

}

fn write_wbspack_header (
	output: &mut Write,
) -> Result <(), String> {

	for header_line in [
		b"WBS PACK 0\0\0\0\0\0\0",
		b"GNU TAR\0\0\0\0\0\0\0\0\0",
	].iter () {

		output.write (
			* header_line);

	}

	Ok (())

}

fn work () -> Result <(), String> {

	let mut stdin =
		std::io::stdin ();

	let mut stdout =
		std::io::stdout ();

	let mut headers =
		Vec::new ();

	let mut block_references =
		Vec::new ();

	try! {
		write_wbspack_header (
			&mut stdout)
	};

	let mut offset = try! (
		write_tar_contents (
			&mut stdin,
			&mut stdout,
			&mut headers,
			&mut block_references)
	);

	offset = try! (
		write_tar_headers (
			& headers,
			&mut stdout,
			offset,
			&mut block_references)
	);

	write_wbspack_footer (
		&mut stdout,
		& block_references);

	Ok (())

}

fn main () {

	match work () {

		Ok (()) => {

			stderr! (
				"All done!");

		},

		Err (error) => {

			stderr! (
				"Error: {}",
				error);

			std::process::exit (1)

		},

	}

}
