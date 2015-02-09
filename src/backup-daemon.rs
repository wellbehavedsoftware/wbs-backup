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

use wbs::backup::state::*;
use wbs::backup::main::*;

mod wbs {

	pub mod backup {

		pub mod log;

		pub mod config;
		pub mod flock;
		pub mod main;
		pub mod run;
		pub mod state;
		pub mod time;

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

	let mut state =
		ProgState::setup (& config_path);

	// run program

	state.write_state ();

	main_loop (&mut state);

	// (never reach here)

}
