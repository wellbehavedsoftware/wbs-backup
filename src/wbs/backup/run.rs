extern crate time;

use std::fs::File;
use std::io::Result;
use std::io::Write;
use std::path::Path;
use std::process;

use time::Timespec;

use wbs::backup::state::*;
use wbs::backup::state::SyncState::*;
use wbs::backup::time::*;

pub fn run_script (
	name: &str,
	script: &str,
	log: &str,
	time: &str,
) -> process::ExitStatus {

	let process_output =
		process::Command::new (script)
		.arg (time)
		.output ()
		.unwrap_or_else (
			|err|
			
			panic! (
				"error running script {}: {}",
				script,
				err)

		);

	let output_path_str =
		format! (
			"{}-{}.log",
			log,
			time);

	let output_path =
		Path::new (& output_path_str);

	let mut output_file =
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

	write_process_output (
		&mut output_file,
		& process_output,
	).unwrap_or_else (
		|err|

		panic! (
			"error writing script output {}: {}",
			script,
			err)

	);

	process_output.status

}

pub fn write_process_output (
	output_file: &mut File,
	process_output: & process::Output,
) -> Result<()> {

	try! {
		write! (
			output_file,
			"STDOUT:\n{:?}\n",
			process_output.stdout)
	}

	try! {
		write! (
			output_file,
			"STDERR:\n{:?}\n",
			process_output.stderr)
	}

	Ok (())

}

pub fn do_sync (
	state: &mut ProgState,
	job_index: usize,
	sync_time: Timespec
) {

	if state.config.jobs [job_index].sync_script.is_some () {

		log! (
			"sync started for {} {}",
			state.config.jobs [job_index].name,
			time_format_pretty (sync_time));

		state.jobs [job_index].state = Syncing;
		state.write_state ();

		let sync_script =
			state.config.jobs [job_index].sync_script.clone ().unwrap ();

		let sync_log =
			state.config.jobs [job_index].sync_log.clone ().unwrap ();

		let exit_status =
			run_script (
				"sync",
				& sync_script,
				& sync_log,
				& time_format_hour (sync_time));

		log! (
			"sync for {} {}",
			state.config.jobs [job_index].name,
			exit_report (exit_status));

		state.jobs [job_index].state = Idle;
		state.jobs [job_index].last_sync = Some (sync_time);

		state.write_state ();

	} else {

		log! (
			"sync skipped for {} {}",
			state.config.jobs [job_index].name,
			time_format_pretty (sync_time));

		state.jobs [job_index].last_sync = Some (sync_time);

		state.write_state ();

	}

}

pub fn do_snapshot (
	state: &mut ProgState,
	job_index: usize,
	snapshot_time: Timespec,
) {

	if state.config.jobs [job_index].snapshot_script.is_some () {

		log! (
			"snapshot started for {} {}",
			state.config.jobs [job_index].name,
			time_format_pretty (snapshot_time));

		state.jobs [job_index].state = Snapshotting;

		state.write_state ();

		let snapshot_script = 
			state.config.jobs [job_index].snapshot_script.clone ().unwrap ();

		let snapshot_log =
			state.config.jobs [job_index].snapshot_log.clone ().unwrap ();

		let exit_status =
			run_script (
				"snapshot",
				& snapshot_script,
				& snapshot_log,
				& time_format_day (snapshot_time));

		log! (
			"snapshot for {} {}",
			state.config.jobs [job_index].name,
			exit_report (exit_status));

		state.jobs [job_index].state = Idle;
		state.jobs [job_index].last_snapshot = Some (snapshot_time);

		state.write_state ();

	} else {

		log! (
			"snapshot skipped for {} {}",
			state.config.jobs [job_index].name,
			time_format_pretty (snapshot_time));

		state.jobs [job_index].last_snapshot = Some (snapshot_time);

		state.write_state ();

	}

}

fn exit_report (
	exit_status: process::ExitStatus,
) -> String {

	if exit_status.success () {

		format! (
			"ended successfully")

	} else {

		match exit_status.code () {

			Some (status) => {

				format! (
					"ended with status {}",
					status)

			},

			None => {

				format! (
					"terminated by signal")

			}

		}

	}

}
