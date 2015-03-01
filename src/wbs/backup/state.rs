extern crate time;

use rustc_serialize::json;

use std::io::Read;
use std::io::Result;
use std::io::Write;

use std::fs;
use std::fs::File;
use std::fs::PathExt;

use std::path::Path;

use time::Timespec;

use wbs::backup::config::*;
use wbs::backup::state::SyncState::*;
use wbs::backup::time::*;

pub enum SyncState {
	Idle,
	Syncing,
	Snapshotting,
	Exporting,
}

impl SyncState {

	fn from_string (str: &str) -> SyncState {

		match str {
			"idle" => { Idle }
			"syncing" => { Syncing }
			"snapshotting" => { Snapshotting }
			"exporting" => { Exporting }
			_ => { panic! ("err") }
		}

	}

}

impl ToString for SyncState {

	fn to_string (& self) -> String {

		match * self {
			Idle => { "idle".to_string () }
			Syncing => { "syncing".to_string () }
			Snapshotting => { "snapshotting".to_string () }
			Exporting => { "exporting".to_string () }
		}

	}

}

#[derive (RustcEncodable, RustcDecodable)]
struct DiskJobState {

	pub name: String,
	pub state: String,

	pub last_sync: Option <String>,
	pub last_snapshot: Option <String>,

}

#[derive (RustcEncodable, RustcDecodable)]
struct DiskState {

	pub jobs: Vec <DiskJobState>,

}

struct JobState {

	pub name: String,
	pub state: SyncState,

	pub last_sync: Option <Timespec>,
	pub last_snapshot: Option <Timespec>,

}

pub struct ProgState {
	pub config: DiskConfig,
	pub jobs: Vec <JobState>,
}

impl ProgState {

	fn read_job_state (
		disk_state: & DiskState,
		job_config: & DiskJobConfig,
	) -> JobState {

		match disk_state.jobs.iter ().find (
			|elem| elem.name == job_config.name
		) {

			None => {
				JobState {
					name: job_config.name.clone (),
					state: Idle,
					last_sync: None,
					last_snapshot: None,
				}
			}

			Some (disk_job_state) => {
				JobState {

					name: job_config.name.clone (),

					state: SyncState::from_string (
						& disk_job_state.state),

					last_sync: time_parse_opt (
						& disk_job_state.last_sync),

					last_snapshot: time_parse_opt (
						& disk_job_state.last_snapshot),

				}
			}

		}

	}

	fn write_job_state (
		job_state: & JobState,
	) -> DiskJobState {

		DiskJobState {
			name: job_state.name.clone (),
			state: job_state.state.to_string (),
			last_sync: time_format_pretty_opt (job_state.last_sync),
			last_snapshot: time_format_pretty_opt (job_state.last_snapshot),
		}

	}

	fn read_state (
		config: DiskConfig,
		state_path: & Path,
	) -> ProgState {

		let mut state_json: String =
			String::new ();

		File::open (
			& state_path,
		).unwrap ().read_to_string (
			&mut state_json,
		).unwrap_or_else (
			|err|

			panic! (
				"error reading state {}: {}",
				state_path.display (),
				err)

		);

		let disk_state: DiskState =
			json::decode (
				& state_json
			).unwrap_or_else (
				|err|

				panic! (
					"error reading state {}: {}",
					state_path.display (),
					err)

			);

		let jobs_temp =
			config.jobs.iter ().map (
				|job_config|

				ProgState::read_job_state (
					& disk_state,
					job_config)

			).collect ();

		ProgState {
			config: config,
			jobs: jobs_temp,
		}

	}

	fn new_state (
		config: DiskConfig,
	) -> ProgState {

		let jobs_temp = config.jobs.iter ().map (
			|job_config|

			JobState {
				name: job_config.name.clone (),
				state: Idle,
				last_sync: None,
				last_snapshot: None,
			}

		).collect ();

		ProgState {
			config: config,
			jobs: jobs_temp,
		}

	}

	pub fn setup (
		config_path: & Path,
	) -> ProgState {

		log! ("loading config");

		let config =
			DiskConfig::read (config_path);

		// load state

		let state_path_str =
			config.state.clone ();

		let state_path =
			Path::new (& state_path_str);

		if state_path.exists () {

			log! ("load existing state");

			ProgState::read_state (
				config,
				state_path,
			)

		} else {

			log! ("no existing state");

			ProgState::new_state (
				config,
			)

		}

	}

	fn write_state_temp (
		&mut self,
		state_path_temp: & Path,
		state_json: &str,
	) -> Result<()> {

		let mut file = try! { File::create (state_path_temp) };

		try! { write! (&mut file, "{}\n", & state_json.to_string ()) }
		try! { file.sync_all () }

		Ok (())

	}

	pub fn write_state (
		&mut self,
	) {

		let disk_state = DiskState {

			jobs: self.jobs.iter ().map (
				|job_state| ProgState::write_job_state (& job_state)
			).collect (),

		};

		let state_json =
			json::encode (& disk_state).unwrap ();

		let state_path_str =
			self.config.state.clone ();

		let state_path =
			Path::new (& state_path_str);

		let state_path_temp_str =
			format! ("{}.temp", self.config.state);

		let state_path_temp =
			Path::new (& state_path_temp_str);

		self.write_state_temp (
			& state_path_temp,
			& state_json,
		).unwrap_or_else (
			|err|

			panic! (
				"error writing state {}: {}",
				state_path_temp.display (),
				err)

		);

		fs::rename (
			&state_path_temp,
			&state_path
		).unwrap_or_else (
			|err|

			panic! (
				"error writing state {}: {}",
				state_path.display (),
				err)

		);

	}

}
