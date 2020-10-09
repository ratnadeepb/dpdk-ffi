/*
 * Created on Tue Oct 06 2020:18:22:44
 * Created by Ratnadeep Bhattacharya
 */

use super::constants::{MAX_CLIENTS, PACKET_READ_SIZE};
use capsule::{Mbuf, Ring};
use capsule_ffi::rte_atomic16_t;
use capsule_ffi::RTE_MAX_ETHPORTS;

/*
 * Define a client structure with all needed info, including
 * stats from the clients.
 */
pub(crate) struct Client {
	rx_q: Ring,
	client_id: u16,
	/* these stats hold how many packets the client will actually receive,
	 * and how many packets were dropped because the client's queue was full.
	 * The port-info stats, in contrast, record how many packets were received
	 * or transmitted on an actual NIC port.
	 */
	stats_rx: u32,
	stats_rx_dropped: u32,
}

impl Client {
	pub(crate) fn get_rx_as_mut(&mut self) -> &mut Ring {
		&mut self.rx_q
	}

	pub(crate) fn get_id(&self) -> u16 {
		self.client_id
	}

	pub(crate) fn set_rx_stat(&mut self, recvd: u32) {
		self.stats_rx += recvd;
	}

	pub(crate) fn set_rx_dropped_stat(&mut self, recvd: u32) {
		self.stats_rx_dropped += recvd;
	}
}

/*
 * Local buffers to put packets in, used to send packets in bursts to the
 * clients
 */
pub(crate) struct ClientRxBuf {
	buffer: Vec<Mbuf>,
	count: u32,
}

impl ClientRxBuf {
	pub(crate) fn new() -> Self {
		Self {
			buffer: Mbuf::alloc_bulk(PACKET_READ_SIZE as usize).unwrap(),
			count: 0,
		}
	}

	pub(crate) fn get_count(&self) -> u32 {
		self.count
	}
	pub(crate) fn get_buff_as_mut(&mut self) -> &mut [Mbuf] {
		&mut self.buffer
	}

	pub(crate) fn increment_count(&mut self) {
		self.count += 1;
	}
}

#[derive(Default)]
pub(crate) struct RxStat {
	rx: [u64; RTE_MAX_ETHPORTS as usize],
}

impl RxStat {
	pub(crate) fn get_rx(&self) -> &[u64] {
		&self.rx
	}

	pub(crate) fn set_rx(&mut self, ind: usize, count: u64) {
		self.rx[ind] += count;
	}
}

#[derive(Default)]
pub(crate) struct TxStat {
	tx: [u64; RTE_MAX_ETHPORTS as usize],
	tx_dropped: [u64; RTE_MAX_ETHPORTS as usize],
}

#[derive(Default)]
pub(crate) struct PortInfo {
	num_ports: u32,
	id: [u16; RTE_MAX_ETHPORTS as usize],
	rx_stats: RxStat,
	tx_drop: [TxStat; RTE_MAX_ETHPORTS as usize],
}

impl PortInfo {
	pub(crate) fn get_num_ports(&self) -> u32 {
		self.num_ports
	}

	pub(crate) fn get_port_id_by_index(&self, ind: usize) -> u16 {
		self.id[ind]
	}

	pub(crate) fn set_port_id_by_index(&mut self, ind: usize, val: u16) {
		self.id[ind] = val;
	}

	pub(crate) fn get_rx_stats(&mut self) -> &mut RxStat {
		&mut self.rx_stats
	}
}

pub enum OnvmAction {
	DROP, // drop packet
	NEXT, // to whatever the next action is configured
	TONF, // // send to the NF specified in the argument field, if on the same host
	OUT,  // send the packet out the NIC port set in the argument field
}

pub struct OnvmPktMeta {
	action: OnvmAction,  // Action to be performed
	destination: u16,    // where to go next
	src: u16,            // who processed the packet last
	pub chain_index: u8, // index of the current step in the service chain
	flags: u8, // bits for custom NF data. Use with caution to prevent collisions from different NFs
}

/// The NF local context will own the NF struct
pub struct OnvmNfLocalCtx {
	nf: OnvmNF,
	nf_init_finished: rte_atomic16_t,
	keep_running: rte_atomic16_t,
	nf_stopped: rte_atomic16_t,
}

/// Define a NF structure with all needed info
/// This structure is available in the NF when processing packets or executing the callback.
/// nf denotes the lifetime of the nf
pub struct OnvmNF {
	pub rx_q: Ring,
	pub tx_q: Ring,
	pub msg_q: Ring,
	pub instance_id: u16,
	pub status: u16,
	pub tag: String,
}
