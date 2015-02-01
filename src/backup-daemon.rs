#![crate_name = "backup-daemon"]
#![crate_type = "bin"]

#![allow(unstable)]

extern crate serialize;
extern crate time;

use flock::Lock;

use serialize::json;

use std::cmp::Ordering;
use std::old_io::File;
use std::old_io::IoResult;
use std::old_io::fs;
use std::old_io::fs::PathExtensions;
use std::old_io::process;
use std::old_io::process::Command;
use std::old_io::process::ProcessExit;
use std::old_io::timer::sleep;
use std::option::*;
use std::os;
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

	fn to_string (&self) -> String {

		match *self {
			Idle => { "idle".to_string () }
			Syncing => { "syncing".to_string () }
			Snapshotting => { "snapshotting".to_string () }
			Exporting => { "exporting".to_string () }
		}

	}

}

#[derive(Encodable,Decodable)]
struct DiskState {

	state: String,
	last_sync: Option<String>,
	last_snapshot: Option<String>,

}

#[derive(Encodable,Decodable)]
struct DiskConfig {

	state: String,

	sync_script: Option<String>,
	sync_log: Option<String>,

	snapshot_script: Option<String>,
	snapshot_log: Option<String>,

}

struct ProgState {

	config: DiskConfig,
	state: SyncState,
	last_sync: Option<Timespec>,
	last_snapshot: Option<Timespec>,

}

impl ProgState {

	fn read_state (
		config_path: &Path
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
				config_json.as_slice ()
			).unwrap_or_else (
				|err|

				panic! (
					"error reading config {}: (todo)",
					config_path.display ())

			);

		let state_path =
			Path::new (&config.state);

		if state_path.exists () {

			log! ("load existing state");

			let state_json =
				File::open (
					&state_path
				).read_to_string ().unwrap_or_else (
					|err|

					panic! (
						"error reading state {}: {}",
						state_path.display (),
						err)

				);

			let state_data: DiskState =
				json::decode (
					state_json.as_slice ()
				).unwrap_or_else (
					|err|

					panic! (
						"error reading state {}: (todo)",
						state_path.display ())

				);

			let new = ProgState {

				config: config,

				state: SyncState::from_string (
					state_data.state.as_slice ()),

				last_sync: state_data.last_sync.map (
					|n| time_parse (n.as_slice ())),

				last_snapshot: state_data.last_snapshot.map (
					|n| time_parse (n.as_slice ())),

			};

			log! ("state: {}", new.state.to_string ());

			log! ("last sync: {}",
				new.last_sync.map_or (
					"none".to_string (),
					|n| time_format_pretty (n)));

			log! ("last snapshot: {}",
				new.last_snapshot.map_or (
					"none".to_string (),
					|n| time_format_pretty (n)));

			return new;

		} else {

			log! ("no existing state");

			return ProgState {
				config: config,
				state: Idle,
				last_sync: None,
				last_snapshot: None,
			};

		}

	}

	fn write_state_temp (
		&mut self,
		state_path_temp: &Path,
		state_json: &str
	) -> IoResult<()> {

		let mut file = try! { File::create (state_path_temp) };
		try! { file.write_str (state_json.to_string ().as_slice ()) }
		try! { file.write_str ("\n") }
		try! { file.fsync () }

		Ok (())

	}

	fn write_state (
		&mut self
	) {

		let disk_state = DiskState {

			state: self.state.to_string (),

			last_sync: self.last_sync.map (
				|n| time_format_pretty (n).to_string ()),

			last_snapshot: self.last_snapshot.map (
				|n| time_format_pretty (n).to_string ()),

		};

		let state_json =
			json::encode (&disk_state).unwrap ();

		let state_path =
			Path::new (self.config.state.clone ());

		let state_path_temp =
			Path::new (format! ("{}.temp", self.config.state));

		self.write_state_temp (
			&state_path_temp,
			state_json.as_slice ()
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

	fn loop_once (&mut self) {

		match self.state {
			Idle => { },
			_ => {

				log! (
					"warning: previous state was {}",
					self.state.to_string ());

				self.state = Idle;

			}
		}

		let now = time::get_time ();
		let last_hour = round_down_hour (now);
		let last_day = round_down_day (now);

		match self.last_sync {
			None => { self.do_sync (last_hour) }
			Some (last_sync) => match last_sync.cmp (&last_hour) {
				Ordering::Less => { self.do_sync (last_hour) }
				Ordering::Equal => { return }
				Ordering::Greater => { panic! ("last sync is in future") }
			}
		}

		match self.last_snapshot {
			None => { self.do_snapshot (last_day) }
			Some (last_snapshot) => match last_snapshot.cmp (&last_day) {
				Ordering::Less => { self.do_snapshot (last_day) }
				Ordering::Equal => { return }
				Ordering::Greater => { panic! ("last snapshot is in future") }
			}
		}

	}

	fn main_loop (&mut self) {
		loop {
			self.loop_once ();
			sleep (Duration::seconds (1));
		}
	}

	fn run_script (
		&self,
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
			File::create (&output_path).unwrap_or_else (
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

	fn do_sync (&mut self, sync_time: Timespec) {

		if self.config.sync_script.is_some () {

			log! (
				"sync started for {}",
				time_format_pretty (sync_time));

			self.state = Syncing;
			self.write_state ();

			let exit_status =
				self.run_script (
					"sync",
					self.config.sync_script.clone ().unwrap ().as_slice (),
					self.config.sync_log.clone ().unwrap ().as_slice (),
					time_format_hour (sync_time).as_slice ());

			log! (
				"sync {}",
				exit_report (exit_status));

			self.state = Idle;
			self.last_sync = Some (sync_time);
			self.write_state ();

		} else {

			log! (
				"sync skipped for {}",
				time_format_pretty (sync_time));

			self.last_sync = Some (sync_time);
			self.write_state ();

		}

	}

	fn do_snapshot (&mut self, snapshot_time: Timespec) {

		if self.config.snapshot_script.is_some () {

			log! (
				"snapshot started for {}",
				time_format_pretty (snapshot_time));

			self.state = Snapshotting;
			self.write_state ();

			let exit_status =
				self.run_script (
					"snapshot",
					self.config.snapshot_script.clone ().unwrap ().as_slice (),
					self.config.snapshot_log.clone ().unwrap ().as_slice (),
					time_format_day (snapshot_time).as_slice ());

			log! (
				"snapshot {}",
				exit_report (exit_status));

			self.state = Idle;
			self.last_snapshot = Some (snapshot_time);
			self.write_state ();

		} else {

			log! (
				"snapshot skipped for {}",
				time_format_pretty (snapshot_time));

			self.last_snapshot = Some (snapshot_time);
			self.write_state ();

		}


	}

}

fn exit_report (process_exit: ProcessExit) -> String {

	match process_exit {

		ProcessExit::ExitStatus (status) => {
			format! ("ended with status {}", status)
		}

		ProcessExit::ExitSignal (signal) => {
			format! ("terminated by signal {}", signal)
		}

	}

}

fn round_down_hour (now: Timespec) -> Timespec {

	Tm {
		tm_min: 0,
		tm_sec: 0,
		tm_nsec: 0,
		..time::at_utc (now)
	}.to_timespec ()

}

fn round_down_day (now: Timespec) -> Timespec {

	Tm {
		tm_hour: 0,
		tm_min: 0,
		tm_sec: 0,
		tm_nsec: 0,
		..time::at_utc (now)
	}.to_timespec ()

}

fn time_format_pretty (when: Timespec) -> String {

	time::strftime (
		"%Y-%m-%d %H:%M:%S",
		&time::at_utc (when)
	).unwrap ()

}

fn time_format_day (when: Timespec) -> String {

	time::strftime (
		"%Y-%m-%d",
		&time::at_utc (when)
	).unwrap ()

}

fn time_format_hour (when: Timespec) -> String {

	time::strftime (
		"%Y-%m-%d-%H",
		&time::at_utc (when)
	).unwrap ()

}

fn time_parse (str: &str) -> Timespec {

	time::strptime (
		str,
		"%Y-%m-%d %H:%M:%S"
	).unwrap ().to_timespec ()

}

fn main () {

	// check args

	let args = os::args ();

	if args.len () != 2 {
		println! ("Syntax error");
		return;
	}

	let config_path =
		Path::new (args [1].clone ());

	// obtain lock
	// TODO move this

	let lock_path = Path::new ("sync-old-server.lock");
	Lock::new (&lock_path);

	// init program

	let mut prog_state =
		ProgState::read_state (&config_path);

	log! ("ready");

	// run program

	prog_state.main_loop ();

	// (never reach here)

}
