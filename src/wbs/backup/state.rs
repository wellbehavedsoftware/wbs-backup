extern crate time;

use rustc_serialize::json;

use std::io::Read;
use std::io::Result;
use std::io::Write;

use std::fs;
use std::fs::File;

use std::path::Path;

use time::Timespec;

use wbs::backup::config::*;
use wbs::backup::time::*;

// ######################################## interface

// ==================== memory state

// ---------- job state

pub enum JobState {
	Idle,
	Syncing,
	Snapshotting,
	Sending,
	Exporting,
}

// ---------- snapshot state

pub enum SnapshotState {
	Snapshotting,
	Snapshotted,
	Sending,
	Sent,
}

// ---------- snapshot

pub struct Snapshot {

	pub state: SnapshotState,

	pub snapshot_time: Timespec,
	pub send_time: Option <Timespec>,

}

// ---------- job

pub struct Job {

	pub name: String,
	pub state: JobState,

	pub last_sync: Option <Timespec>,
	pub last_snapshot: Option <Timespec>,
	pub last_send: Option <Timespec>,

	pub snapshots: Vec <Snapshot>,

}

// ---------- global

pub struct Global {
	pub jobs: Vec <Job>,
}

// ==================== disk state

#[derive (RustcEncodable, RustcDecodable)]
struct DiskSnapshot {

	pub state: String,

	pub snapshot_time: String,
	pub send_time: Option <String>,

}

#[derive (RustcEncodable, RustcDecodable)]
struct DiskJob {

	pub name: String,
	pub state: String,

	pub last_sync: Option <String>,
	pub last_snapshot: Option <String>,
	pub last_send: Option <String>,

	pub snapshots: Option <Vec <DiskSnapshot>>,

}

#[derive (RustcEncodable, RustcDecodable)]
struct DiskState {

	pub jobs: Vec <DiskJob>,

}

// ######################################## implementation

// ==================== memory state

// ---------- job state

impl JobState {

	fn from_string (str: &str) -> JobState {

		match str {
			"idle" => { JobState::Idle }
			"syncing" => { JobState::Syncing }
			"snapshotting" => { JobState::Snapshotting }
			"sending" => { JobState::Sending }
			"exporting" => { JobState::Exporting }
			_ => { panic! ("err") }
		}

	}

}

impl ToString for JobState {

	fn to_string (& self) -> String {

		match * self {
			JobState::Idle => { "idle".to_string () }
			JobState::Syncing => { "syncing".to_string () }
			JobState::Snapshotting => { "snapshotting".to_string () }
			JobState::Sending => { "sending".to_string () }
			JobState::Exporting => { "exporting".to_string () }
		}

	}

}

// ---------- snapshot state

impl SnapshotState {

	fn from_string (str: &str) -> SnapshotState {

		match str {
			"snapshotting" => { SnapshotState::Snapshotting }
			"snapshotted" => { SnapshotState::Snapshotted }
			"sending" => { SnapshotState::Sending }
			"sent" => { SnapshotState::Sent }
			_ => { panic! ("err") }
		}

	}

}

impl ToString for SnapshotState {

	fn to_string (& self) -> String {

		match * self {
			SnapshotState::Snapshotting => { "snapshotting".to_string () }
			SnapshotState::Snapshotted => { "snapshotted".to_string () }
			SnapshotState::Sending => { "sending".to_string () }
			SnapshotState::Sent => { "sent".to_string () }
		}

	}

}

// ---------- global state

impl Global {

	fn read_job (
		disk_state: & DiskState,
		job_config: & JobConfig,
	) -> Job {

		match disk_state.jobs.iter ().find (
			|elem| elem.name == job_config.name
		) {

			None => {

				Job {
					name: job_config.name.clone (),
					state: JobState::Idle,
					last_sync: None,
					last_snapshot: None,
					last_send: None,
					snapshots: vec! [],
				}

			}

			Some (disk_job) => {

				Job {

					name: job_config.name.clone (),

					state: JobState::from_string (
						& disk_job.state),

					last_sync: time_parse_opt (
						& disk_job.last_sync),

					last_snapshot: time_parse_opt (
						& disk_job.last_snapshot),

					last_send: time_parse_opt (
						& disk_job.last_send),

					snapshots: match & disk_job.snapshots {

						& Some (ref disk_snapshots) => {

							disk_snapshots.iter ().map (
								|disk_snapshot|

								Global::read_snapshot (
									disk_snapshot)

							).collect ()

						},

						& None => vec! [],

					}

				}

			}

		}

	}

	fn read_snapshot (
		disk_snapshot: & DiskSnapshot,
	) -> Snapshot {

		Snapshot {

			state: SnapshotState::from_string (
				& disk_snapshot.state),

			snapshot_time: time_parse (
				& disk_snapshot.snapshot_time),

			send_time: time_parse_opt (
				& disk_snapshot.send_time),

		}

	}

	fn write_job (
		job: & Job,
	) -> DiskJob {

		DiskJob {

			name: job.name.clone (),
			state: job.state.to_string (),

			last_sync: time_format_pretty_opt (
				job.last_sync),

			last_snapshot: time_format_pretty_opt (
				job.last_snapshot),

			last_send: time_format_pretty_opt (
				job.last_send),

			snapshots: Some (job.snapshots.iter ().map (
				|snapshot|

				Global::write_snapshot (
					snapshot)

			).collect ()),

		}

	}

	fn write_snapshot (
		snapshot: & Snapshot,
	) -> DiskSnapshot {

		DiskSnapshot {

			state: snapshot.state.to_string (),

			snapshot_time: time_format_pretty (
				snapshot.snapshot_time),

			send_time: time_format_pretty_opt (
				snapshot.send_time),

		}

	}

	fn read_state (
		config: & Config,
		state_path: & Path,
	) -> Global {

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

				Global::read_job (
					& disk_state,
					job_config)

			).collect ();

		Global {
			jobs: jobs_temp,
		}

	}

	fn new_state (
		config: & Config,
	) -> Global {

		let jobs_temp = config.jobs.iter ().map (
			|job_config|

			Job {
				name: job_config.name.clone (),
				state: JobState::Idle,
				last_sync: None,
				last_snapshot: None,
				last_send: None,
				snapshots: vec! [],
			}

		).collect ();

		Global {
			jobs: jobs_temp,
		}

	}

	pub fn read (
		config: & Config,
	) -> Global {

		// load state

		let state_path_str =
			config.state.clone ();

		let state_path =
			Path::new (& state_path_str);

		match fs::metadata (state_path) {

			Ok (_) => {

				log! ("load existing state");

				Global::read_state (
					config,
					state_path,
				)

			},

			Err (_) => {

				log! ("no existing state");

				Global::new_state (
					config,
				)

			},

		}

	}

	fn write_state_temp (
		& self,
		state_path_temp: & Path,
		state_json: &str,
	) -> Result<()> {

		let mut file = try! { File::create (state_path_temp) };

		try! { write! (&mut file, "{}\n", & state_json.to_string ()) }
		try! { file.sync_all () }

		Ok (())

	}

	pub fn write_state (
		& self,
		config: & Config,
	) {

		let disk_state = DiskState {

			jobs: self.jobs.iter ().map (
				|job_state|

				Global::write_job (
					& job_state,
				)

			).collect (),

		};

		let state_json =
			json::encode (& disk_state).unwrap ();

		let state_path_str =
			config.state.clone ();

		let state_path =
			Path::new (& state_path_str);

		let state_path_temp_str =
			format! ("{}.temp", config.state);

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
