extern crate time;

use time::Timespec;
use time::Tm;

pub fn round_down_hour (
	now: Timespec,
) -> Timespec {

	Tm {
		tm_min: 0,
		tm_sec: 0,
		tm_nsec: 0,
		..time::at_utc (now)
	}.to_timespec ()

}

pub fn round_down_day (
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

pub fn time_format_pretty (
	when: Timespec,
) -> String {

	time::strftime (
		"%Y-%m-%d %H:%M:%S",
		&time::at_utc (when),
	).unwrap ()

}

pub fn time_format_pretty_opt (
	when_opt: Option <Timespec>,
) -> Option<String> {

	match when_opt {
		None => None,
		Some (when) => Some (time_format_pretty (when)),
	}

}

pub fn time_format_day (
	when: Timespec,
) -> String {

	time::strftime (
		"%Y-%m-%d",
		& time::at_utc (when),
	).unwrap ()

}

pub fn time_format_hour (
	when: Timespec,
) -> String {

	time::strftime (
		"%Y-%m-%d-%H",
		& time::at_utc (when),
	).unwrap ()

}

pub fn time_parse (str: &str) -> Timespec {

	time::strptime (
		str,
		"%Y-%m-%d %H:%M:%S",
	).unwrap ().to_timespec ()

}

pub fn time_parse_opt (
	opt_str: & Option <String>
) -> Option <Timespec> {

	match opt_str {
		& None => { None },
		& Some (ref val) => { Some (time_parse (& val)) },
	}

}
