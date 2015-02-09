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

use std::env;

use wbs::backup::state::ProgState;

mod wbs {
	pub mod backup {
		mod config;
		mod flock;
		mod log;
		pub mod state;
		mod time;
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

	// run program

	prog_state.write_state ();

	prog_state.main_loop ();

	// (never reach here)

}
