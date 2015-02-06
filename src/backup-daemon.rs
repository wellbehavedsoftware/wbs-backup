#![crate_name = "backup-daemon"]
#![crate_type = "bin"]

#![feature (core)]
#![feature (env)]
#![feature (io)]
#![feature (libc)]
#![feature (os)]
#![feature (path)]
#![feature (std_misc)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate time;

use flock::Lock;

use rustc_serialize::json;

use std::cmp::Ordering;
use std::env;
use std::old_io::File;
use std::old_io::IoResult;
use std::old_io::fs;
use std::old_io::fs::PathExtensions;
use std::old_io::process;
use std::old_io::process::Command;
use std::old_io::process::ProcessExit;
use std::old_io::timer::sleep;
use std::option::*;
use std::os::unix::prelude::*;
use std::time::Duration;

use time::Timespec;
use time::Tm;

use SyncState::*;

mod flock;

macro_rules! log {

	($($arg:tt)*) => {

		println! (
			"{}: {}",
			time_format_pretty (time::get_time ()),
			format! ($($arg)*))

	}

}

enum SyncState {
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

		match *self {
			Idle => { "idle".to_string () }
			Syncing => { "syncing".to_string () }
			Snapshotting => { "snapshotting".to_string () }
			Exporting => { "exporting".to_string () }
		}

	}

}

#[derive (RustcEncodable, RustcDecodable)]
struct DiskJobState {

	name: String,
	state: String,

	last_sync: Option <String>,
	last_snapshot: Option <String>,

}

#[derive (RustcEncodable, RustcDecodable)]
struct DiskState {

	jobs: Vec <DiskJobState>,

}

#[derive (RustcEncodable, RustcDecodable)]
struct DiskJobConfig {

	name: String,

	sync_script: Option <String>,
	sync_log: Option <String>,

	snapshot_script: Option <String>,
	snapshot_log: Option <String>,

}

#[derive (RustcEncodable, RustcDecodable)]
struct DiskConfig {

	state: String,
	lock: String,

	jobs: Vec <DiskJobConfig>,

}

struct JobState {

	name: String,
	state: SyncState,

	last_sync: Option <Timespec>,
	last_snapshot: Option <Timespec>,

}

struct ProgState {

	config: DiskConfig,

	lock: Lock,

	jobs: Vec <JobState>,

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

	fn setup (
		config_path: & Path,
	) -> ProgState {

		log! ("loading config");

		let config_json =
			File::open (
				&config_path
			).read_to_string ().unwrap_or_else (
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

		// obtain lock

		log! ("obtaining lock");

		let lock_path =
			Path::new (config.lock.clone ());

		let lock =
			Lock::new (& lock_path, false);

		// load state

		let state_path =
			Path::new (& config.state);

		if state_path.exists () {

			log! ("load existing state");

			let state_json =
				File::open (
					& state_path,
				).read_to_string ().unwrap_or_else (
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

			let new = ProgState {
				config: config,
				lock: lock,
				jobs: jobs_temp,
			};

			return new;

		} else {

			log! ("no existing state");

			let jobs_temp = config.jobs.iter ().map (
				|job_config|

				JobState {
					name: job_config.name.clone (),
					state: Idle,
					last_sync: None,
					last_snapshot: None,
				}

			).collect ();

			return ProgState {
				config: config,
				lock: lock,
				jobs: jobs_temp,
			};

		}

	}

	fn write_state_temp (
		&mut self,
		state_path_temp: & Path,
		state_json: &str,
	) -> IoResult<()> {

		let mut file = try! { File::create (state_path_temp) };

		try! { file.write_str (& state_json.to_string ()) }
		try! { file.write_str ("\n") }
		try! { file.fsync () }

		Ok (())

	}

	fn write_state (
		&mut self,
	) {

		let disk_state = DiskState {

			jobs: self.jobs.iter ().map (
				|job_state| ProgState::write_job_state (& job_state)
			).collect (),

		};

		let state_json =
			json::encode (& disk_state).unwrap ();

		let state_path =
			Path::new (self.config.state.clone ());

		let state_path_temp =
			Path::new (format! ("{}.temp", self.config.state));

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

	fn loop_job (
		&mut self,
		job_index: usize,
	) {

		let now = time::get_time ();
		let last_hour = round_down_hour (now);
		let last_day = round_down_day (now);

		match self.jobs [job_index].last_sync {
			None => { self.do_sync (job_index, last_hour) }
			Some (last_sync) => match last_sync.cmp (&last_hour) {
				Ordering::Less => { self.do_sync (job_index, last_hour) }
				Ordering::Equal => { return }
				Ordering::Greater => { panic! ("last sync is in future") }
			}
		}

		match self.jobs [job_index].last_snapshot {
			None => { self.do_snapshot (job_index, last_day) }
			Some (last_snapshot) => match last_snapshot.cmp (&last_day) {
				Ordering::Less => { self.do_snapshot (job_index, last_day) }
				Ordering::Equal => { return }
				Ordering::Greater => { panic! ("last snapshot is in future") }
			}
		}

	}

	fn loop_once (&mut self) {

		for i in 0 .. self.jobs.len () {
			self.loop_job (i)
		}

	}

	fn main_loop (&mut self) {
		loop {
			self.loop_once ();
			sleep (Duration::seconds (1));
		}
	}

	fn run_script (
		& self,
		name: &str,
		script: &str,
		log: &str,
		time: &str
	) -> ProcessExit {

		let output_path =
			Path::new (format! (
				"{}-{}.log",
				log,
				time));

		let output_file =
			File::create (
				& output_path
			).unwrap_or_else (
				|err| 
				
				panic! (
					"error creating {} log {}: {}",
					name,
					output_path.display (),
					err)
			
			);

		let mut process =
			Command::new (script)
			.arg (time)
			.stdin (process::Ignored)
			.stdout (process::InheritFd (output_file.as_raw_fd ()))
			.stderr (process::InheritFd (output_file.as_raw_fd ()))
			.spawn ()
			.unwrap_or_else (
				|err|
				
				panic! (
					"error running script {}: {}",
					script,
					err)

			);

		process.wait ().unwrap_or_else (
			|err|
			
			panic! (
				"error running script {}: {}",
				script,
				err)

		)

	}

	fn do_sync (
		&mut self,
		job_index: usize,
		sync_time: Timespec
	) {

		if self.config.jobs [job_index].sync_script.is_some () {

			log! (
				"sync started for {} {}",
				self.config.jobs [job_index].name,
				time_format_pretty (sync_time));

			self.jobs [job_index].state = Syncing;
			self.write_state ();

			let exit_status =
				self.run_script (
					"sync",
					& self.config.jobs [job_index].sync_script.clone ().unwrap (),
					& self.config.jobs [job_index].sync_log.clone ().unwrap (),
					& time_format_hour (sync_time));

			log! (
				"sync for {} {}",
				self.config.jobs [job_index].name,
				exit_report (exit_status));

			self.jobs [job_index].state = Idle;
			self.jobs [job_index].last_sync = Some (sync_time);

			self.write_state ();

		} else {

			log! (
				"sync skipped for {} {}",
				self.config.jobs [job_index].name,
				time_format_pretty (sync_time));

			self.jobs [job_index].last_sync = Some (sync_time);

			self.write_state ();

		}

	}

	fn do_snapshot (
		&mut self,
		job_index: usize,
		snapshot_time: Timespec,
	) {

		if self.config.jobs [job_index].snapshot_script.is_some () {

			log! (
				"snapshot started for {} {}",
				self.config.jobs [job_index].name,
				time_format_pretty (snapshot_time));

			self.jobs [job_index].state = Snapshotting;

			self.write_state ();

			let exit_status =
				self.run_script (
					"snapshot",
					& self.config.jobs [job_index].snapshot_script.clone ().unwrap (),
					& self.config.jobs [job_index].snapshot_log.clone ().unwrap (),
					& time_format_day (snapshot_time));

			log! (
				"snapshot for {} {}",
				self.config.jobs [job_index].name,
				exit_report (exit_status));

			self.jobs [job_index].state = Idle;
			self.jobs [job_index].last_snapshot = Some (snapshot_time);

			self.write_state ();

		} else {

			log! (
				"snapshot skipped for {} {}",
				self.config.jobs [job_index].name,
				time_format_pretty (snapshot_time));

			self.jobs [job_index].last_snapshot = Some (snapshot_time);

			self.write_state ();

		}

	}

}

fn exit_report (
	process_exit: ProcessExit,
) -> String {

	match process_exit {

		ProcessExit::ExitStatus (status) => {

			format! (
				"ended with status {}",
				status)

		}

		ProcessExit::ExitSignal (signal) => {

			format! (
				"terminated by signal {}",
				signal)

		}

	}

}

fn round_down_hour (
	now: Timespec,
) -> Timespec {

	Tm {
		tm_min: 0,
		tm_sec: 0,
		tm_nsec: 0,
		..time::at_utc (now)
	}.to_timespec ()

}

fn round_down_day (
	now: Timespec,
) -> Timespec {

	Tm {
		tm_hour: 0,
		tm_min: 0,
		tm_sec: 0,
		tm_nsec: 0,
		..time::at_utc (now)
	}.to_timespec ()

}

fn time_format_pretty (
	when: Timespec,
) -> String {

	time::strftime (
		"%Y-%m-%d %H:%M:%S",
		&time::at_utc (when),
	).unwrap ()

}

fn time_format_pretty_opt (
	when_opt: Option <Timespec>,
) -> Option<String> {

	match when_opt {
		None => None,
		Some (when) => Some (time_format_pretty (when)),
	}

}

fn time_format_day (
	when: Timespec,
) -> String {

	time::strftime (
		"%Y-%m-%d",
		& time::at_utc (when),
	).unwrap ()

}

fn time_format_hour (
	when: Timespec,
) -> String {

	time::strftime (
		"%Y-%m-%d-%H",
		& time::at_utc (when),
	).unwrap ()

}

fn time_parse (str: &str) -> Timespec {

	time::strptime (
		str,
		"%Y-%m-%d %H:%M:%S",
	).unwrap ().to_timespec ()

}

fn time_parse_opt (
	opt_str: & Option <String>
) -> Option <Timespec> {

	match opt_str {
		& None => { None },
		& Some (ref val) => { Some (time_parse (& val)) },
	}

}

fn main () {

	// check args

	let args: Vec <String> =
		env::args ().map (
			|os_str| os_str.into_string ().unwrap ()
		).collect ();

	if args.len () != 2 {
		println! ("Syntax error");
		return;
	}

	let config_path =
		Path::new (args [1].clone ());

	// init program

	let mut prog_state =
		ProgState::setup (& config_path);

	log! ("ready");

	// run program

	prog_state.write_state ();

	prog_state.main_loop ();

	// (never reach here)

}
