#![crate_name = "backup_daemon"]
#![crate_type = "bin"]

extern crate rustc_serialize;
extern crate time;

use std::env;

use std::path::Path;

use wbs::backup::config::*;
use wbs::backup::state::*;
use wbs::backup::main::*;

mod wbs {

	pub mod backup {

		pub mod log;

		pub mod config;
		pub mod main;
		pub mod run;
		pub mod state;
		pub mod time;

	}

}

fn main () {

	// check args

	let args: Vec <String> =
		env::args ().collect ();

	if args.len () != 2 {
		println! ("Syntax error");
		return;
	}

	let config_path_str =
		args [1].clone ();

	let config_path =
		Path::new (& config_path_str);

	// init program

	let config =
		Config::read (& config_path);

	let mut state =
		Global::read (& config);

	// run program

	state.write_state (& config);

	main_loop (& config, &mut state);

	// (never reach here)

}
