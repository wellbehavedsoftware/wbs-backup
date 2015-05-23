extern crate time;

use rustc_serialize::json;

use std::io::Read;
use std::fs::File;
use std::path::Path;

use wbs::backup::time::*;

#[derive (RustcEncodable, RustcDecodable)]
pub struct JobConfig {

	pub name: String,

	pub sync_script: Option <String>,
	pub sync_log: Option <String>,

	pub snapshot_script: Option <String>,
	pub snapshot_log: Option <String>,

	pub send_script: Option <String>,
	pub send_log: Option <String>,

}

#[derive (RustcEncodable, RustcDecodable)]
pub struct Config {

	pub state: String,
	pub lock: String,

	pub jobs: Vec <JobConfig>,

}

impl Config {

	pub fn read (
		config_path: & Path,
	) -> Config {

		log! ("loading config");

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

		let config: Config =
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
