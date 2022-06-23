// use anyhow::{bail, Result};
// use libc::{c_char, c_short, ioctl, socket, IF_NAMESIZE};
// use std::io::Error;

// #[repr(C)]
// pub struct IfReq {
// 	ifr_name: [c_char; IF_NAMESIZE],
// 	union: [u8; 24],
// }

// impl Default for IfReq {
// 	fn default() -> Self {
// 		let name: [c_char; IF_NAMESIZE] = [0 as c_char; IF_NAMESIZE];
// 		IfReq {
// 			ifr_name: name,
// 			union: [0; 24],
// 		}
// 	}
// }
// impl IfReq {
// 	pub fn ifr_flags(&mut self, flags: c_short) {
// 		// Zero the flags and copy the two bytes of flags
// 		self.union = [0; 24];
// 		self.union[0] = flags as u8;
// 		self.union[1] = (flags << 8) as u8;
// 	}

// 	pub fn from_name(name: &str) -> Option<Self> {
// 		if name.len() >= IF_NAMESIZE {
// 			None
// 		} else {
// 			let mut ifreq: IfReq = IfReq::default();
// 			for (i, c) in name.as_bytes().iter().enumerate() {
// 				ifreq.ifr_name[i] = *c as c_char;
// 			}
// 			Some(ifreq)
// 		}
// 	}

// 	/// Tells the kernel to set the interface flags from this request.
// 	/// # Safety
// 	/// This function uses ioctl behind the scenes, therefore requiring an unsafe function.
// 	pub unsafe fn write_flags(&self) -> Result<i32> {
// 		let fd = socket(libc::AF_INET, libc::SOCK_DGRAM, 0);

// 		if fd < 0 {
// 			bail!(
// 				"Failed to set interface flags due to socket error: {}",
// 				Error::last_os_error()
// 			);
// 		}

// 		#[cfg(target_env = "musl")]
// 		{
// 			let res = ioctl(fd, libc::SIOCSIFFLAGS.try_into().unwrap(), &self);

// 			if res < 0 {
// 				bail!(
// 					"Failed to set interface flags: {}",
// 					Error::last_os_error()
// 				);
// 			}

// 			Ok(res)
// 		}

// 		#[cfg(not(target_env = "musl"))]
// 		{
// 			let res = ioctl(fd, libc::SIOCSIFFLAGS, &self);

// 			if res < 0 {
// 				bail!(
// 					"Failed to set interface flags: {}",
// 					Error::last_os_error()
// 				);
// 			}

// 			Ok(res)
// 		}
// 	}
// }
