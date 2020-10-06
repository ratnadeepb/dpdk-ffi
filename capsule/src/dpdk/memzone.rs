/*
 * Created on Tue Oct 06 2020:13:05:36
 * Created by Ratnadeep Bhattacharya
 */

// Deep: my attempt at a wrapper implementation for DPDK rings
use super::SocketId;
use crate::dpdk::DpdkError;
use crate::ffi::{self, AsStr, ToCString, ToResult};
use crate::{debug, info};
use failure::Fallible;
// use std::cell::Cell;
// use std::collections::HashMap;
use std::fmt;
use std::os::raw;
// use std::ptr::{self, NonNull};
// use std::sync::atomic::{AtomicUsize, Ordering};

const NO_FLAGS: u8 = 0;

/// A physical memory zone reserved
/// rte_memzone.addr is the start of the virtual memory address
pub struct Memzone {
	raw: *const ffi::rte_memzone, // memzone_reserve returns *const so NonNull not required (see ToResult trait)
}

impl Memzone {
	/// Creates a new `Memzone`.
	///
	/// `capacity` is the The size of the memory to be reserved.
	/// If it is 0, the biggest contiguous zone will be reserved.
	///
	/// `socket_id` is the socket where the memory should be allocated. The
	/// value can be `SocketId::ANY` if there is no constraint.
	///
	/// # Errors
	///
	/// If allocation fails, then `DpdkError` is returned.
	pub fn new(name: String, capacity: usize, socket_id: SocketId) -> Fallible<Self> {
		let raw = unsafe {
			ffi::rte_memzone_reserve(
				name.clone().to_cstring().as_ptr(),
				capacity as u64,
				socket_id.raw(),
				NO_FLAGS as raw::c_uint,
			)
			.to_result(|_| DpdkError::new())?
		};
		info!("created {}", name);
		Ok(Self { raw })
	}

	/// Returns the raw struct needed for FFI calls.
	#[inline]
	pub fn raw(&self) -> &ffi::rte_memzone {
		unsafe { &*self.raw }
	}

	/// Returns the name of the `Mempool`.
	#[inline]
	pub fn name(&self) -> &str {
		self.raw().name[..].as_str()
	}
}

impl fmt::Debug for Memzone {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let raw = self.raw();
		unsafe {
			f.debug_struct(self.name())
				.field("capacity", &raw.len)
				.field("hugepage_size", &raw.hugepage_sz)
				.field("flags", &format_args!("{:#x}", raw.flags))
				.field("socket", &raw.socket_id)
				.finish()
		}
	}
}

impl Drop for Memzone {
	fn drop(&mut self) {
		debug!("freeing {}.", self.name());
		unsafe {
			ffi::rte_memzone_free(self.raw());
		}
	}
}
