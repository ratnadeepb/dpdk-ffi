/*
 * Created on Tue Oct 06 2020:02:06:53
 * Created by Ratnadeep Bhattacharya
 */

// Deep: my attempt at a wrapper implementation for DPDK rings
use super::SocketId;
use crate::dpdk::DpdkError;
use crate::ffi::{self, AsStr, ToCString, ToResult};
use crate::{debug, info};
use failure::{Fail, Fallible};
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;
use std::os::raw;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A memory pool is an allocator of message buffers, or `Mbuf`. For best
/// performance, each socket should have a dedicated `Mempool`.
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
	/// `cache_size` is the per core object cache. If cache_size is non-zero,
	/// the library will try to limit the accesses to the common lockless
	/// pool. The cache can be disabled if the argument is set to 0.
	///
	/// `socket_id` is the socket where the memory should be allocated. The
	/// value can be `SocketId::ANY` if there is no constraint.
	///
	/// # Errors
	///
	/// If allocation fails, then `DpdkError` is returned.
	pub fn new(
		name: String,
		capacity: usize,
		cache_size: usize,
		socket_id: SocketId,
	) -> Fallible<Self> {
		
	}
}
