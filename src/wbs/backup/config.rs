use rustc_serialize::json;

use std::io::Read;
use std::fs::File;
use std::path::Path;

#[derive (RustcEncodable, RustcDecodable)]
pub struct DiskJobConfig {

	pub name: String,

	pub sync_script: Option <String>,
	pub sync_log: Option <String>,

	pub snapshot_script: Option <String>,
	pub snapshot_log: Option <String>,

}

#[derive (RustcEncodable, RustcDecodable)]
pub struct DiskConfig {

	pub state: String,
	pub lock: String,

	pub jobs: Vec <DiskJobConfig>,

}

impl DiskConfig {

	pub fn read (
		config_path: & Path,
	) -> DiskConfig {

		let mut config_json: String =
			String::new ();

		File::open (
			& config_path,
		).unwrap ().read_to_string (
			&mut config_json,
		).unwrap_or_else (
			|err|

			panic! (
				"error reading config {}: {}",
				config_path.display (),
				err)

		);

		let config: DiskConfig =
			json::decode (
				& config_json,
			).unwrap_or_else (
				|err|

				panic! (
					"error reading config {}: {}",
					config_path.display (),
					err)

			);

		config

	}

}
