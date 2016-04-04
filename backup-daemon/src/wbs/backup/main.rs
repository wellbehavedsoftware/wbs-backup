extern crate time;

use std::cmp::Ordering;
use std::thread;
use std::time::Duration;

use wbs::backup::config::*;
use wbs::backup::run::*;
use wbs::backup::state::*;
use wbs::backup::time::*;

fn loop_job (
	config: & Config,
	state: &mut Global,
	job_index: usize,
) {

	let now = time::get_time ();
	let last_hour = round_down_hour (now);
	let last_day = round_down_day (now);

	match state.jobs [job_index].last_sync {

		None => {
			do_sync (
				config,
				state,
				job_index,
				last_hour,
			)
		}

		Some (last_sync) => match last_sync.cmp (& last_hour) {

			Ordering::Less => {
				do_sync (
					config,
					state,
					job_index,
					last_hour,
				)
			}

			Ordering::Equal => {
				return
			}

			Ordering::Greater => {
				panic! ("last sync is in future")
			}

		}

	}

	match state.jobs [job_index].last_snapshot {

		None => {
			do_snapshot (
				config,
				state,
				job_index,
				last_day,
			)
		}

		Some (last_snapshot) => match last_snapshot.cmp (&last_day) {

			Ordering::Less => {
				do_snapshot (
					config,
					state,
					job_index,
					last_day,
				)
			}

			Ordering::Equal => {
				return
			}

			Ordering::Greater => {
				panic! ("last snapshot is in future")
			}

		}

	}

	match state.jobs [job_index].last_send {

		None => {
			do_send (
				config,
				state,
				job_index,
				last_day,
			)
		}

		Some (last_send) => match last_send.cmp (&last_day) {

			Ordering::Less => {
				do_send (
					config,
					state,
					job_index,
					last_day,
				)
			}

			Ordering::Equal => {
				return
			}

			Ordering::Greater => {
				panic! ("last send is in future")
			}

		}

	}

}

fn loop_once (
	config: & Config,
	state: &mut Global,
) {

	for i in 0 .. state.jobs.len () {
		loop_job (config, state, i)
	}

}

pub fn main_loop (
	config: & Config,
	state: &mut Global,
) {

	loop {

		loop_once (config, state);

		thread::sleep (
			Duration::from_millis (1000));

	}

}
