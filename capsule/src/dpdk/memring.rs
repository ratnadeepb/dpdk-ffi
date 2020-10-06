/*
 * Created on Tue Oct 06 2020:02:06:53
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
use std::ptr::NonNull;

const NO_FLAGS: u8 = 0;

/// A ring is intended to communicate between two DPDK processes by sending/receiving `Mbuf`.
/// For best performance, each socket should have a dedicated `Mempool`.
pub struct Ring {
	raw: NonNull<ffi::rte_ring>,
}

impl Ring {
	/// Creates a new `Ring` for `Mbuf`.
	///
	/// `capacity` is the maximum number of `Mbuf` the `Mempool` can hold.
	/// The optimum size (in terms of memory usage) is when n is a power
	/// of two minus one.
	///
	/// `socket_id` is the socket where the memory should be allocated. The
	/// value can be `SocketId::ANY` if there is no constraint.
	///
	/// # Errors
	///
	/// If allocation fails, then `DpdkError` is returned.
	pub fn new(name: String, capacity: usize, socket_id: SocketId) -> Fallible<Self> {
		let raw = unsafe {
			ffi::rte_ring_create(
				name.clone().to_cstring().as_ptr(),
				capacity as raw::c_uint,
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
	pub fn raw(&self) -> &ffi::rte_ring {
		unsafe { self.raw.as_ref() }
	}

	/// Returns the raw struct needed for FFI calls.
	#[inline]
	pub fn raw_mut(&mut self) -> &mut ffi::rte_ring {
		unsafe { self.raw.as_mut() }
	}

	/// Returns the name of the `Mempool`.
	#[inline]
	pub fn name(&self) -> &str {
		self.raw().name[..].as_str()
	}
}

impl fmt::Debug for Ring {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let raw = self.raw();
		unsafe {
			f.debug_struct(self.name())
				.field("capacity", &raw.capacity)
				.field("flags", &format_args!("{:#x}", raw.flags))
				.finish()
		}
	}
}

impl Drop for Ring {
	fn drop(&mut self) {
		debug!("freeing {}.", self.name());
		unsafe {
			ffi::rte_ring_free(self.raw_mut());
		}
	}
}
