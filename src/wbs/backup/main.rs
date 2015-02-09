extern crate time;

use std::cmp::Ordering;
use std::old_io::timer::sleep;
use std::time::Duration;

use wbs::backup::run::*;
use wbs::backup::state::*;
use wbs::backup::time::*;

fn loop_job (
	state: &mut ProgState,
	job_index: usize,
) {

	let now = time::get_time ();
	let last_hour = round_down_hour (now);
	let last_day = round_down_day (now);

	match state.jobs [job_index].last_sync {

		None => {
			do_sync (
				state,
				job_index,
				last_hour,
			)
		}

		Some (last_sync) => match last_sync.cmp (&last_hour) {

			Ordering::Less => {
				do_sync (
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
				state,
				job_index,
				last_day,
			)
		}

		Some (last_snapshot) => match last_snapshot.cmp (&last_day) {

			Ordering::Less => {
				do_snapshot (
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

}

fn loop_once (
	state: &mut ProgState,
) {

	for i in 0 .. state.jobs.len () {
		loop_job (state, i)
	}

}

pub fn main_loop (
	state: &mut ProgState,
) {

	loop {
		loop_once (state);
		sleep (Duration::seconds (1));
	}

}
