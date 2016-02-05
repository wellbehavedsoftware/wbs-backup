extern crate time;

use std::fs::File;
use std::io::Result;
use std::io::Write;
use std::path::Path;
use std::process;

use time::Timespec;

use wbs::backup::config::*;
use wbs::backup::state::*;
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

	try! (write! (
		output_file,
		"STDOUT:\n"));

	try! (output_file.write (
		& process_output.stdout));

	try! (write! (
		output_file,
		"\nSTDERR:\n"));

	try! (output_file.write (
		& process_output.stderr));

	try! (write! (
		output_file,
		"\n"));

	Ok (())

}

pub fn do_sync (
	config: & Config,
	state: &mut Global,
	job_index: usize,
	sync_time: Timespec
) {

	let job_config = & config.jobs [job_index];

	if job_config.sync_script.is_some () {

		log! (
			"sync started for {} {}",
			job_config.name,
			time_format_pretty (sync_time));

		state.jobs [job_index].state =
			JobState::Syncing;

		state.write_state (config);

		let sync_script =
			job_config.sync_script.clone ().unwrap ();

		let sync_log =
			job_config.sync_log.clone ().unwrap ();

		let exit_status =
			run_script (
				"sync",
				& sync_script,
				& sync_log,
				& time_format_hour (sync_time));

		log! (
			"sync for {} {}",
			job_config.name,
			exit_report (exit_status));

		state.jobs [job_index].state =
			JobState::Idle;

		state.jobs [job_index].last_sync =
			Some (sync_time);

		state.write_state (config);

	} else {

		log! (
			"sync skipped for {} {}",
			job_config.name,
			time_format_pretty (sync_time));

		state.jobs [job_index].last_sync =
			Some (sync_time);

		state.write_state (config);

	}

}

pub fn do_snapshot (
	config: & Config,
	state: &mut Global,
	job_index: usize,
	snapshot_time: Timespec,
) {

	let job_config = & config.jobs [job_index];

	if job_config.snapshot_script.is_some () {

		log! (
			"snapshot started for {} {}",
			job_config.name,
			time_format_pretty (snapshot_time));

		state.jobs [job_index].state =
			JobState::Snapshotting;

		let snapshot_index =
			state.jobs [job_index].snapshots.len ();

		state.jobs [job_index].snapshots.push (
			Snapshot {
				state: SnapshotState::Snapshotting,
				snapshot_time: snapshot_time,
				send_time: None,
			}
		);

		state.write_state (config);

		let snapshot_script =
			job_config.snapshot_script.clone ().unwrap ();

		let snapshot_log =
			job_config.snapshot_log.clone ().unwrap ();

		let exit_status =
			run_script (
				"snapshot",
				& snapshot_script,
				& snapshot_log,
				& time_format_day (snapshot_time));

		log! (
			"snapshot for {} {}",
			job_config.name,
			exit_report (exit_status));

		state.jobs [job_index].state =
			JobState::Idle;

		state.jobs [job_index].last_snapshot =
			Some (snapshot_time);

		state.jobs [job_index].snapshots [snapshot_index].state =
			SnapshotState::Snapshotted;

		state.write_state (config);

	} else {

		log! (
			"snapshot skipped for {} {}",
			job_config.name,
			time_format_pretty (snapshot_time));

		state.jobs [job_index].last_snapshot =
			Some (snapshot_time);

		state.write_state (config);

	}

}

pub fn do_send (
	config: & Config,
	state: &mut Global,
	job_index: usize,
	send_time: Timespec,
) {

	let job_config = & config.jobs [job_index];

	if job_config.send_script.is_some () {

		log! (
			"send started for {} {}",
			job_config.name,
			time_format_pretty (send_time));

		let mut snapshot_indexes: Vec <usize> = vec! [];

		for (snapshot_index, snapshot)
			in state.jobs [job_index].snapshots.iter ().enumerate () {

			match snapshot.state {

				SnapshotState::Snapshotted => {

					snapshot_indexes.push (snapshot_index)

				},

				_ => {},

			}

		}

		for snapshot_index in snapshot_indexes {

			do_send_snapshot (
				config,
				state,
				job_index,
				snapshot_index,
				send_time);

		}

	} else {

		log! (
			"send skipped for {} {}",
			job_config.name,
			time_format_pretty (send_time));

		state.jobs [job_index].last_send =
			Some (send_time);

		state.write_state (config);

	}

}

pub fn do_send_snapshot (
	config: & Config,
	state: &mut Global,
	job_index: usize,
	snapshot_index: usize,
	send_time: Timespec,
) {

	let job_config = & config.jobs [job_index];

	if job_config.send_script.is_some () {

		log! (
			"send started for {} {}",
			job_config.name,
			time_format_pretty (send_time));

		state.jobs [job_index].state =
			JobState::Sending;

		state.jobs [job_index].snapshots [snapshot_index].state =
			SnapshotState::Sending;

		state.jobs [job_index].snapshots [snapshot_index].send_time =
			Some (send_time);

		state.write_state (config);

		let send_script =
			job_config.send_script.clone ().unwrap ();

		let send_log =
			job_config.send_log.clone ().unwrap ();

		let exit_status =
			run_script (
				"send",
				& send_script,
				& send_log,
				& time_format_day (send_time));

		log! (
			"send completed for {} {}",
			job_config.name,
			exit_report (exit_status));

		state.jobs [job_index].state =
			JobState::Idle;

		state.jobs [job_index].last_send =
			Some (send_time);

		state.jobs [job_index].snapshots [snapshot_index].state =
			SnapshotState::Sent;

		state.write_state (config);

	} else {

		log! (
			"send skipped for {} {}",
			job_config.name,
			time_format_pretty (send_time));

		state.jobs [job_index].last_send =
			Some (send_time);

		state.write_state (config);

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
