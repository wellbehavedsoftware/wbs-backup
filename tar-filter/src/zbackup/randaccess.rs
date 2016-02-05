use protobuf;
use protobuf::stream::CodedInputStream;

use rustc_serialize::hex::ToHex;

use std::io;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use misc::*;
use zbackup::proto;
use zbackup::repo::ZBackup;

enum InstructionRefContent {
	Chunk ([u8; 24]),
	Bytes (Vec <u8>),
}

struct InstructionRef {

	content: InstructionRefContent,

	start: u64,
	end: u64,

}

pub struct RandomAccess <'a> {

	repo: & 'a mut ZBackup,
	instruction_refs: Vec <InstructionRef>,
	size: u64,

	position: u64,
	chunk_cursor: Cursor <Vec <u8>>,

}

impl <'a> RandomAccess <'a> {

	pub fn new (
		repo: & 'a mut ZBackup,
		backup_name: & str,
	) -> Result <RandomAccess <'a>, TfError> {

		let mut input =
			Cursor::new (
				try! (
					repo.read_and_expand_backup (
						backup_name)));

		let mut coded_input_stream =
			CodedInputStream::new (
				&mut input);

		let mut instruction_refs: Vec <InstructionRef> =
			vec! ();

		let mut offset: u64 = 0;

		while ! try! (coded_input_stream.eof ()) {

			let instruction_length =
				try! (
					coded_input_stream.read_raw_varint32 ());

			let instruction_old_limit =
				try! (
					coded_input_stream.push_limit (
						instruction_length));

			let backup_instruction =
				try! (
					protobuf::core::parse_from::<proto::BackupInstruction> (
						&mut coded_input_stream));

			coded_input_stream.pop_limit (
				instruction_old_limit);

			if backup_instruction.has_chunk_to_emit () {

				let chunk_id =
					to_array (
						backup_instruction.get_chunk_to_emit ());

				let index_entry =
					try! (
						repo.get_index_entry (
							& chunk_id));

				instruction_refs.push (
					InstructionRef {

					content:
						InstructionRefContent::Chunk (
							to_array (
								backup_instruction.get_chunk_to_emit ())),

					start:
						offset,

					end:
						offset + index_entry.size,

				});

				offset +=
					index_entry.size;

			}

			if backup_instruction.has_bytes_to_emit () {

				let bytes =
					backup_instruction.get_bytes_to_emit ();

				let size =
					bytes.len () as u64;

				instruction_refs.push (
					InstructionRef {

					content:
						InstructionRefContent::Bytes (
							bytes.to_owned ()),

					start:
						offset,

					end:
						offset + size,

				});

				offset +=
					size;

			}

		}

		Ok (RandomAccess {

			repo: repo,
			instruction_refs: instruction_refs,
			size: offset,

			position: 0,
			chunk_cursor: Cursor::new (vec! []),

		})

	}

}

impl <'a> Read for RandomAccess <'a> {

	fn read (
		& mut self,
		buffer: & mut [u8],
	) -> Result <usize, io::Error> {

		let mut function_bytes_read: u64 = 0;

		loop {

			let loop_bytes_read: u64 =
				try! (
					self.chunk_cursor.read (
						& mut buffer [ function_bytes_read as usize .. ])
				) as u64;

			self.position +=
				loop_bytes_read;

			function_bytes_read +=
				loop_bytes_read;

			if function_bytes_read == buffer.len () as u64 {

				break;

			}

			match self.instruction_refs.iter ().find (
				|instruction_ref|
				instruction_ref.start >= self.position
				&& self.position < instruction_ref.end,
			) {

				Some (instruction_ref) => match instruction_ref.content {

					InstructionRefContent::Chunk (chunk_id) => {

						let chunk_bytes =
							try! (
								self.repo.get_chunk (
									chunk_id,
								).map_err (
									|_error|
									io::Error::new (
										io::ErrorKind::InvalidData,
										format! (
											"Chunk not found: {}",
											chunk_id.to_hex ()))
								));

						self.chunk_cursor =
							Cursor::new (
								chunk_bytes.to_owned ());

					},

					InstructionRefContent::Bytes (ref bytes_data) => {

						self.chunk_cursor =
							Cursor::new (
								bytes_data.to_owned ());

					},

				},

				None => {
					break;
				},

			}

		}

		Ok (
			function_bytes_read as usize)

	}

}

impl <'a> Seek for RandomAccess <'a> {

	fn seek (
		& mut self,
		seek_from: SeekFrom,
	) -> io::Result <u64> {

		self.position =
			match seek_from {

			SeekFrom::Start (value) =>
				value,

			SeekFrom::Current (value) =>
				((self.position as i64) + value) as u64,

			SeekFrom::End (value) =>
				((self.size as i64) + value) as u64,

		};

		match self.instruction_refs.iter ().find (
			|instruction_ref|
			instruction_ref.start <= self.position
			&& self.position < instruction_ref.end,
		) {

			Some (instruction_ref) => match instruction_ref.content {

				InstructionRefContent::Chunk (chunk_id) => {

					let chunk_bytes =
						try! (
							self.repo.get_chunk (
								chunk_id,
							).map_err (
								|_error|
								io::Error::new (
									io::ErrorKind::InvalidData,
									format! (
										"Chunk not found: {}",
										chunk_id.to_hex ()))
							));

					self.chunk_cursor =
						Cursor::new (
							chunk_bytes.to_owned ());

					try! (
						self.chunk_cursor.seek (
							SeekFrom::Start (
								self.position - instruction_ref.start)));

				},

				InstructionRefContent::Bytes (ref bytes_data) => {

					self.chunk_cursor =
						Cursor::new (
							bytes_data.to_owned ());

					try! (
						self.chunk_cursor.seek (
							SeekFrom::Start (
								self.position - instruction_ref.start)));

				},

			},

			None => {


				self.chunk_cursor =
					Cursor::new (vec! []);

			},

		}

		Ok (self.position)

	}

}
