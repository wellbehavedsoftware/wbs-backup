#[ doc (hidden) ]
#[ macro_export ]
macro_rules! stderr {

	( $ ( $arg : tt ) * ) => (

		match write! (
			&mut ::std::io::stderr () as &mut ::std::io::Write,
			$ ( $arg ) *,
		) {

			Ok (_) => {},

			Err (error) => panic! (
				"Unable to write to stderr: {}",
				error),

		}

	)

}

#[ doc (hidden) ]
#[ macro_export ]
macro_rules! stderrln {

	( $ ( $arg : tt ) * ) => (

		match writeln! (
			&mut ::std::io::stderr () as &mut ::std::io::Write,
			$ ( $arg ) *,
		) {

			Ok (_) => {},

			Err (error) => panic! (
				"Unable to write to stderr: {}",
				error),

		}

	)

}

// ex: noet ts=4 filetype=rust
