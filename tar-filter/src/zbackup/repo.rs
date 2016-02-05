use protobuf;
use protobuf::stream::CodedInputStream;

use rustc_serialize::hex::ToHex;

use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;

use misc::*;
use zbackup::proto;
use zbackup::read::*;

pub struct ZBackup {
	path: String,
	master_index: HashMap <[u8; 24], [u8; 24]>,
	chunk_cache: HashMap <[u8; 24], Vec <u8>>,
}

impl ZBackup {

	pub fn open (
		repository_path: &str,
	) -> Result <ZBackup, TfError> {

		// load info file

		stderrln! (
			"Loading repository {}",
			repository_path);

		let _storage_info =
			try! (
				read_storage_info (
					& format! (
						"{}/info",
						repository_path)));

		// load indexes

		stderr! (
			"Loading indexes");

		let mut master_index: HashMap <[u8; 24], [u8; 24]> =
			HashMap::with_capacity (0x10000);

		let mut count: u64 = 0;

		for dir_entry_or_error in try! (
			fs::read_dir (
				format! (
					"{}/index",
					repository_path))
		) {

			let dir_entry =
				try! (
					dir_entry_or_error);

			let file_name =
				dir_entry.file_name ();

			let index_name =
				file_name.to_str ().unwrap ();

			let index =
				try! (
					read_index (
						& format! (
							"{}/index/{}",
							repository_path,
							index_name)));

			for (index_bundle_header, bundle_info) in index {

				for chunk_record in bundle_info.get_chunk_record () {

					master_index.insert (
						to_array (chunk_record.get_id ()),
						to_array (index_bundle_header.get_id ()));

				}

			}

			if count & 0x3f == 0x3f {
				stderr! (
					".");
			}

			count += 1;

		}

		stderr! (
			"\n");

		// return

		Ok (ZBackup {
			path: repository_path.to_string (),
			master_index: master_index,
			chunk_cache: HashMap::new (),
		})

	}

	pub fn restore (
		& mut self,
		backup_name: & str,
		output: & mut Write,
	) -> Result <(), TfError> {

		// load backup

		stderr! (
			"Loading backup {}",
			backup_name);

		let backup_info =
			try! (
				read_backup_file (
					format! (
						"{}/backups/{}",
						& self.path,
						backup_name)));

		// expand backup data

		let mut input =
			Cursor::new (
				backup_info.get_backup_data ().to_owned ());

		for _iteration in 0 .. backup_info.get_iterations () {

			let mut temp_output: Cursor <Vec <u8>> =
				Cursor::new (
					Vec::new ());

			try! (
				self.follow_instructions (
					& mut input,
					& mut temp_output,
					& |count| {
						if count & 0xf == 0xf {
							stderr! (".");
						}
					}));

			input =
				Cursor::new (
					temp_output.into_inner ());

		}

		stderr! (
			"\n");

		// restore backup

		stderr! (
			"Restoring backup");

		try! (
			self.follow_instructions (
				& mut input,
				output,
				& |count| {
					if count & 0x1ff == 0x1ff {
						stderr! (".");
					}
				}));

		stderr! (
			"\n");

		stderrln! (
			"Restore complete");

		Ok (())

	}

	pub fn follow_instructions (
		& mut self,
		input: & mut Read,
		output: & mut Write,
		progress: & Fn (u64),
	) -> Result <(), TfError> {

		let mut coded_input_stream =
			CodedInputStream::new (
				input);

		let mut count: u64 = 0;

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

				let chunk_data = try! (
					self.get_chunk (
						chunk_id));

				try! (
					output.write (
						& chunk_data));

			}

			if backup_instruction.has_bytes_to_emit () {

				try! (
					output.write (
						backup_instruction.get_bytes_to_emit ()));

			}

			progress (
				count);

			count += 1;

		}

		Ok (())

	}

	pub fn get_chunk (
		& mut self,
		chunk_id: [u8; 24],
	) -> Result <& Vec <u8>, TfError> {

		if ! self.chunk_cache.contains_key (& chunk_id) {

			if self.chunk_cache.len () >= 0x1000 {

				self.chunk_cache.clear ();

			}

			if ! self.master_index.contains_key (& chunk_id) {
				panic! ();
			}

			let & bundle_id =
				self.master_index.get (& chunk_id).unwrap ();

			for (found_chunk_id, found_chunk_data) in try! (
				read_bundle (
					& format! (
						"{}/bundles/{}/{}",
						self.path,
						& bundle_id.to_hex () [0 .. 2],
						bundle_id.to_hex ()))
			) {

				self.chunk_cache.insert (
					found_chunk_id,
					found_chunk_data);

			}

		}

		let chunk_data =
			self.chunk_cache.get (
				& chunk_id,
			).unwrap ();

		Ok (chunk_data)

	}

}
