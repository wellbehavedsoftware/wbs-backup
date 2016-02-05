use protobuf::core;
use protobuf::stream;

use rustc_serialize::hex::ToHex;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path;
use std::io;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

use lzma;
use misc::*;
use zbackup_proto;

pub struct ZBackup {
	path: String,
	master_index: HashMap <[u8; 24], [u8; 24]>,
	chunk_cache: HashMap <[u8; 24], Vec <u8>>,
}

pub type IndexEntry = (
	zbackup_proto::IndexBundleHeader,
	zbackup_proto::BundleInfo,
);

impl ZBackup {

	pub fn open (
		path: &str,
	) -> Result <ZBackup, TfError> {

		// load info file

		let _storage_info =
			try! (
				ZBackup::read_storage_info (
					& format! (
						"{}/info",
						path)));

		// load indexes

		try! (
			io::stderr ().write (
				"Loading indexes".as_bytes ()));

		let mut master_index: HashMap <[u8; 24], [u8; 24]> =
			HashMap::with_capacity (0x10000);

		for dir_entry_or_error in try! (
			fs::read_dir (
				format! (
					"{}/index",
					path))
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
					ZBackup::read_index (
						& format! (
							"{}/index/{}",
							path,
							index_name)));

			for (index_bundle_header, bundle_info) in index {

				for chunk_record in bundle_info.get_chunk_record () {

					master_index.insert (
						to_array (chunk_record.get_id ()),
						to_array (index_bundle_header.get_id ()));

				}

			}

			try! (
				io::stderr ().write (
					".".as_bytes ()));

		}

		try! (
			io::stderr ().write (
				"\n".as_bytes ()));

		// return

		Ok (ZBackup {
			path: path.to_string (),
			master_index: master_index,
			chunk_cache: HashMap::new (),
		})

	}

	pub fn read_storage_info (
		path: &str,
	) -> Result <zbackup_proto::StorageInfo, TfError> {

		// open file

		let mut input =
			try! (
				File::open (
					path));

		let mut coded_input_stream =
			stream::CodedInputStream::new (
				&mut input);

		// read header

		let header_length =
			try! (
				coded_input_stream.read_raw_varint32 ());

		let header_old_limit =
			try! (
				coded_input_stream.push_limit (
					header_length));

		let file_header =
			try! (
				core::parse_from::<zbackup_proto::FileHeader> (
					&mut coded_input_stream));

		if file_header.get_version () != 1 {

			panic! (
				"Unsupported backup version {}",
				file_header.get_version ());

		}

		coded_input_stream.pop_limit (
			header_old_limit);

		// read storage info

		let storage_info_length =
			try! (
				coded_input_stream.read_raw_varint32 ());

		let storage_info_old_limit =
			try! (
				coded_input_stream.push_limit (
					storage_info_length));

		let storage_info =
			try! (
				core::parse_from::<zbackup_proto::StorageInfo> (
					&mut coded_input_stream));

		coded_input_stream.pop_limit (
			storage_info_old_limit);

		Ok (storage_info)

	}

	pub fn restore (
		& mut self,
		backup_name: & str,
		output: & mut Write,
	) -> Result <(), TfError> {

		let backup_info =
			try! (
				self.read_backup_file (
					format! (
						"{}/backups/{}",
						& self.path,
						backup_name)));

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
					& mut temp_output));

			input =
				Cursor::new (
					temp_output.into_inner ());

		}

		// read backup instructions 3

		try! (
			self.follow_instructions (
				& mut input,
				output));

		Ok (())

	}

	pub fn follow_instructions (
		& mut self,
		input: & mut Read,
		output: & mut Write,
	) -> Result <(), TfError> {

		let mut coded_input_stream =
			stream::CodedInputStream::new (
				input);

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
					core::parse_from::<zbackup_proto::BackupInstruction> (
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
				ZBackup::read_bundle (
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

	pub fn read_backup_file <P: AsRef <path::Path>> (
		& self,
		path: P,
	) -> Result <zbackup_proto::BackupInfo, TfError> {

		// open file

		let mut input =
			try! (
				File::open (
					path));

		let mut coded_input_stream =
			stream::CodedInputStream::new (
				&mut input);

		// read header

		let header_length =
			try! (
				coded_input_stream.read_raw_varint32 ());

		let header_old_limit =
			try! (
				coded_input_stream.push_limit (
					header_length));

		let file_header =
			try! (
				core::parse_from::<zbackup_proto::FileHeader> (
					&mut coded_input_stream));

		if file_header.get_version () != 1 {

			panic! (
				"Unsupported backup version {}",
				file_header.get_version ());

		}

		coded_input_stream.pop_limit (
			header_old_limit);

		// read backup info

		let backup_info_length =
			try! (
				coded_input_stream.read_raw_varint32 ());

		let backup_info_old_limit =
			try! (
				coded_input_stream.push_limit (
					backup_info_length));

		let backup_info =
			try! (
				core::parse_from::<zbackup_proto::BackupInfo> (
					&mut coded_input_stream));

		coded_input_stream.pop_limit (
			backup_info_old_limit);

		Ok (backup_info)

	}

	pub fn read_index (
		path: &str,
	) -> Result <Vec <IndexEntry>, TfError> {

		let mut index_entries: Vec <IndexEntry> =
			vec! ();

		// open file

		let mut input =
			try! (
				File::open (
					path));

		let mut coded_input_stream =
			stream::CodedInputStream::new (
				&mut input);

		// read header

		let header_length =
			try! (
				coded_input_stream.read_raw_varint32 ());

		let header_old_limit =
			try! (
				coded_input_stream.push_limit (
					header_length));

		let file_header =
			try! (
				core::parse_from::<zbackup_proto::FileHeader> (
					&mut coded_input_stream));

		if file_header.get_version () != 1 {

			panic! (
				"Unsupported backup version {}",
				file_header.get_version ());

		}

		coded_input_stream.pop_limit (
			header_old_limit);

		loop {

			// read index bundle header

			let index_bundle_header_length =
				try! (
					coded_input_stream.read_raw_varint32 ());

			let index_bundle_header_old_limit =
				try! (
					coded_input_stream.push_limit (
						index_bundle_header_length));

			let index_bundle_header =
				try! (
					core::parse_from::<zbackup_proto::IndexBundleHeader> (
						&mut coded_input_stream));

			coded_input_stream.pop_limit (
				index_bundle_header_old_limit);

			if ! index_bundle_header.has_id () {
				break;
			}

			// read bundle info

			let bundle_info_length =
				try! (
					coded_input_stream.read_raw_varint32 ());

			let bundle_info_old_limit =
				try! (
					coded_input_stream.push_limit (
						bundle_info_length));

			let bundle_info =
				try! (
					core::parse_from::<zbackup_proto::BundleInfo> (
						&mut coded_input_stream));

			coded_input_stream.pop_limit (
				bundle_info_old_limit);

			index_entries.push ( (
				index_bundle_header,
				bundle_info) );

		}

		Ok (index_entries)

	}	

	pub fn read_bundle (
		path: &str,
	) -> Result <Vec <([u8; 24], Vec <u8>)>, TfError> {

		// open file

		let input =
			try! (
				File::open (
					path));

		let mut buf_input =
			BufReader::new (
				input);

		let bundle_info = {

			let mut coded_input_stream =
				stream::CodedInputStream::from_buffered_reader (
					&mut buf_input);

			// read header

			let header_length =
				try! (
					coded_input_stream.read_raw_varint32 ());

			let header_old_limit =
				try! (
					coded_input_stream.push_limit (
						header_length));

			let file_header =
				try! (
					core::parse_from::<zbackup_proto::FileHeader> (
						&mut coded_input_stream));

			if file_header.get_version () != 1 {

				panic! (
					"Unsupported backup version {}",
					file_header.get_version ());

			}

			coded_input_stream.pop_limit (
				header_old_limit);

			// read bundle infos

			let bundle_info_length =
				try! (
					coded_input_stream.read_raw_varint32 ());

			let bundle_info_old_limit =
				try! (
					coded_input_stream.push_limit (
						bundle_info_length));

			let bundle_info =
				try! (
					core::parse_from::<zbackup_proto::BundleInfo> (
						&mut coded_input_stream));

			coded_input_stream.pop_limit (
				bundle_info_old_limit);

			bundle_info

		};

		// skip checksum

		try! (
			buf_input.seek (
				SeekFrom::Current (4)));

		// decode compressed data

		let mut chunks: Vec <([u8; 24], Vec <u8>)> =
			vec! {};

		let mut lzma_reader =
			try! (
				lzma::LzmaReader::new (
					& mut buf_input));

		// split into chunks

		for chunk_record in bundle_info.get_chunk_record () {

			let mut chunk_bytes: Vec <u8> =
				Vec::with_capacity (
					chunk_record.get_size () as usize);

			unsafe {

				chunk_bytes.set_len (
					chunk_record.get_size () as usize);

			}

			try! (
				lzma_reader.read_exact (
					&mut chunk_bytes));

			chunks.push (
				(
					to_array (chunk_record.get_id ()),
					chunk_bytes,
				)
			);

		}

		Ok (chunks)

	}

}

pub fn to_array (
	slice: & [u8],
) -> [u8; 24] {

	[
		slice [0],  slice [1],  slice [2],  slice [3],  slice [4],  slice [5],
		slice [6],  slice [7],  slice [8],  slice [9],  slice [10], slice [11],
		slice [12], slice [13], slice [14], slice [15], slice [16], slice [17],
		slice [18], slice [19], slice [20], slice [21], slice [22], slice [23],
	]

}
