/*
 * Created on Tue Oct 06 2020:02:01:56
 * Created by Ratnadeep Bhattacharya
 */

use capsule_ffi::RTE_MAX_ETHPORTS;

/// define common names for structures shared between server and NF
pub(crate) const PKTMBUF_POOL_NAME: &str = "MProc_pktmbuf_pool";
pub(crate) const MZ_PORT_INFO: &str = "MProc_port_info";
pub const SWITCH_TO_NF_RING_NAME: &str = "SWITCH_TO_NF_RING";
pub const NF_TO_SWITCH_RING_NAME: &str = "NF_TO_SWITCH_RING";
pub const SHARED_MEMPOOL_NAME: &str = "SHARED_MEMPOOL";
/*
 * When doing reads from the NIC or the client queues,
 * use this batch size
 */
pub(crate) const PACKET_READ_SIZE: u8 = 32;

/* Number of ports on our switch */
pub(crate) const MAX_CLIENTS: u32 = RTE_MAX_ETHPORTS;

pub(crate) const MBUF_CACHE_SIZE: u32 = 512;
pub(crate) const RTE_MP_RX_DESC_DEFAULT: u32 = 1024;
pub(crate) const RTE_MP_TX_DESC_DEFAULT: u32 = 1024;
pub(crate) const CLIENT_QUEUE_RINGSIZE: u32 = 128;
pub(crate) const NO_FLAGS: u8 = 0;
