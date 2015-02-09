#![allow (non_camel_case_types)]

pub use self::imp::Lock;

mod imp {

	extern crate libc;

	use std::ffi::CString;

	mod os {

		extern crate libc;

		pub struct flock {
			pub l_type: libc::c_short,
			pub l_whence: libc::c_short,
			pub l_start: libc::off_t,
			pub l_len: libc::off_t,
			pub l_pid: libc::pid_t,
		}

		pub const F_WRLCK: libc::c_short = 1;
		pub const F_UNLCK: libc::c_short = 2;
		pub const F_SETLK: libc::c_int = 6;
		pub const F_SETLKW: libc::c_int = 7;

	}

	pub struct Lock {
		fd: libc::c_int,
	}

	impl Lock {

		pub fn new (
			path: & Path,
			wait: bool,
		) -> Lock {

			let buf =
				CString::from_slice (path.as_vec ());

			let fd = unsafe {

				libc::open (
					buf.as_ptr (),
					libc::O_RDWR | libc::O_CREAT,
					libc::S_IRUSR | libc::S_IWUSR)

			};

			assert! (fd > 0);

			let flock = os::flock {
				l_start: 0,
				l_len: 0,
				l_pid: 0,
				l_whence: libc::SEEK_SET as libc::c_short,
				l_type: os::F_WRLCK,
			};

			let ret = unsafe {

				libc::fcntl (
					fd,
					if wait { os::F_SETLKW } else { os::F_SETLK },
					& flock as * const os::flock)

			};

			if ret == -1 {

				unsafe { libc::close (fd); }

				panic! (
					"could not lock '{}'",
					path.display ())

			}

			Lock { fd: fd }

		}

	}

	impl Drop for Lock {

		fn drop (
			&mut self,
		) {

			let flock = os::flock {
				l_start: 0,
				l_len: 0,
				l_pid: 0,
				l_whence: libc::SEEK_SET as libc::c_short,
				l_type: os::F_UNLCK,
			};

			unsafe {

				libc::fcntl (
					self.fd,
					os::F_SETLK,
					& flock as * const os::flock,
				);

				libc::close (self.fd);

			}

		}

	}

}
