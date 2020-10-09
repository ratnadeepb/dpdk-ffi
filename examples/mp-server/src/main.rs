/*
 * Created on Tue Oct 06 2020:22:21:44
 * Created by Ratnadeep Bhattacharya
 */

mod constants;
mod onvm_nf;

// DPDK functions
use capsule::dpdk::{eal_cleanup, eal_init};
use capsule_ffi::{
    _rte_eth_rx_burst, _rte_pktmbuf_free, _rte_ring_enqueue_bulk, rte_calloc,
    rte_eth_dev_count_avail, rte_log, rte_pktmbuf_pool_create, rte_socket_id, stop_and_close_ports,
};
use ctrlc;
use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr;

// DPDK structs
// use capsule::dpdk::{Mempool, Memzone, Ring, SocketId};
use capsule::{Mbuf, Mempool, Memzone, PortBuilder, Ring, SocketId};
use capsule_ffi::rte_mbuf;

// DPDK constants
use capsule_ffi::{RTE_LOGTYPE_USER1, RTE_LOG_INFO, RTE_MAX_ETHPORTS, RTE_MBUF_DEFAULT_BUF_SIZE};

use capsule::config::{load_config, RuntimeConfig};

use constants::{
    CLIENT_QUEUE_RINGSIZE, MAX_CLIENTS, MBUF_CACHE_SIZE, NF_TO_SWITCH_RING_NAME, NO_FLAGS,
    PACKET_READ_SIZE, PKTMBUF_POOL_NAME, RTE_MP_RX_DESC_DEFAULT, RTE_MP_TX_DESC_DEFAULT,
    SHARED_MEMPOOL_NAME, SWITCH_TO_NF_RING_NAME,
};
use onvm_nf::{Client, ClientRxBuf, PortInfo};

fn get_config() -> RuntimeConfig {
    let config = load_config().unwrap();
    config
}

fn main() {
    /* ==================== global variables ==================== */
    let mut pktmbuf_pool: Mempool;
    /* One buffer per client rx queue - dynamically allocate array */
    let mut clients: Vec<Client> = Vec::with_capacity(MAX_CLIENTS as usize);
    let mut num_clients = clients.len();
    let mut cl_rx_buf: Vec<ClientRxBuf> = Vec::with_capacity(num_clients);
    let mut ports: PortInfo = Default::default();
    /* ==================== global variables ==================== */

    /* handling CTRL + C */
    if let Err(e) = ctrlc::set_handler(move || {
        unsafe { stop_and_close_ports() };
    }) {
        panic!("{:?}", e);
    }

    let config = load_config().unwrap();
    let dpdk_args = config.to_eal_args();

    // let args = std::env::args().collect::<Vec<String>>();

    // println!("args: {:?}", args);
    // let mut all_args = args.split(|elem| elem == "--");
    // let mut _s;
    // let dpdk_args = match all_args.next() {
    //     Some(s) => s,
    //     None => {
    //         _s = [("".to_string())];
    //         &_s
    //     }
    // };
    println!("dpdk_args: {:?}", dpdk_args);
    // let onvm_args = match all_args.next() {
    //     Some(s) => s,
    //     None => {
    //         _s = [("".to_string())];
    //         &_s
    //     }
    // };
    // println!("onvm_args: {:?}", onvm_args);

    /* initialise the environment */
    // if let Err(_) = eal_init(args) {
    if let Err(_) = eal_init(dpdk_args) {
        if let Err(e) = eal_cleanup() {
            panic!("{:?}", e);
        }
    };
    // println!("EAL started successfully");
    unsafe {
        rte_log(
            RTE_LOG_INFO,
            RTE_LOGTYPE_USER1,
            "EAL started successfully\n" as *const _ as *const c_char,
        )
    };

    /* get total number of ports */
    let total_ports = unsafe { rte_eth_dev_count_avail() };
    println!("Got total ports: {}", total_ports);

    let socket_id = SocketId::current();

    /* Run the switch */
    do_packet_forwarding(&mut ports, &mut clients, &mut cl_rx_buf, num_clients);
}

/// Initialise the mbuf pool for packet reception for the NIC, and any other
/// buffer pools needed by the app - currently none.
fn init_mbuf_pools(ports: &mut PortInfo, pktmbuf_pool: &mut Mempool, num_clients: usize) {
    let num_mbufs_server = RTE_MP_RX_DESC_DEFAULT * ports.get_num_ports();
    let num_mbufs_client = num_clients as u32
        * (CLIENT_QUEUE_RINGSIZE + RTE_MP_TX_DESC_DEFAULT * ports.get_num_ports());
    let num_mbufs_mp_cache = (num_clients as u32 + 1) * MBUF_CACHE_SIZE;
    let num_mbufs = num_mbufs_server + num_mbufs_client + num_mbufs_mp_cache;

    /* don't pass single-producer/single-consumer flags to mbuf create as it
     * seems faster to use a cache instead */
    println!(
        "Creating mbuf pool '{}' [{} mbufs] ...",
        PKTMBUF_POOL_NAME, num_mbufs
    );

    let socket_id = SocketId::current();
    *pktmbuf_pool = Mempool::new(
        PKTMBUF_POOL_NAME.into(),
        num_mbufs as usize,
        MBUF_CACHE_SIZE as usize,
        socket_id,
    )
    .unwrap();
}

// REVIEW: We want the clients to send their packets back to the server
// The server will send the packets out
// So, Tx and Rx queues will be set up on the server alone
// In this case, we can use capsule::PortQueue which will always provide a paired Rx and Tx queue
// along with receive and transmit functions
// REVIEW: This design also means that we need two mempools - one for Rx and one for Tx
// REVIEW: PortBuilder uses Mempools local to sockets for Ports, so all relevant mempools should be initialised first
/// Initialise an individual port:
/// - configure number of rx and tx rings
/// - set up each rx ring to pull from the main Rx mbuf pool
/// - set up each tx ring to put in the main Tx mbuf pool
fn init_port() {}

/// send a burst of traffic to a client, assuming there are packets
/// available to be sent to this client
fn flush_rx_queue(client: u16, clients: &mut Vec<Client>, cl_rx_buf: &mut Vec<ClientRxBuf>) {
    if cl_rx_buf[client as usize].get_count() == 0 {
        return;
    }
    let cl = &mut clients[client as usize];
    let count = cl_rx_buf[client as usize].get_count();
    if unsafe {
        _rte_ring_enqueue_bulk(
            cl.get_rx_as_mut().raw_mut(),
            &mut (cl_rx_buf[client as usize].get_buff_as_mut() as *mut _ as *mut c_void),
            // cl_rx_buf[client as usize].get_count(),
            count,
            ptr::null_mut(),
        )
    } == 0
    {
        // enqueue failed
        for j in 0..count as usize {
            unsafe { _rte_pktmbuf_free(cl_rx_buf[client as usize].get_buff_as_mut()[j].raw_mut()) };
        }
        cl.set_rx_dropped_stat(count);
    } else {
        // enqueue succeded
        cl.set_rx_stat(count);
    }
}

/// marks a packet down to be sent to a particular client process
fn enqueue_rx_packet(client: u16, buf: Mbuf, cl_rx_buf: &mut Vec<ClientRxBuf>) {
    let cl = &mut cl_rx_buf[client as usize];
    cl.increment_count();
    let count = cl.get_count() as usize; // immutable borrow has to end
    cl.get_buff_as_mut()[count] = buf; // before mutable borrow can happen
}

/// this function will ultimately become the full blown switch
/// right now it simply forwards packets to all clients in a round robing manner
fn process_pkts(
    port_num: u32,
    mut pkts: Vec<Mbuf>,
    rx_count: u16,
    clients: &mut Vec<Client>,
    cl_rx_buf: &mut Vec<ClientRxBuf>,
    num_clients: usize,
) {
    let mut client = 0_u16;
    let mut pkt = pkts.drain(..); // a draining iterator to move packets out
                                  // this discussion can help understand the problem - https://users.rust-lang.org/t/cannot-move-out-of-mutable-reference/35660
    for _ in 0..rx_count as usize {
        enqueue_rx_packet(client, pkt.next().unwrap(), cl_rx_buf);
        if (client + 1) as usize == num_clients {
            client = 0
        } else {
            client += 1;
        }
    }
    for i in 0..num_clients {
        flush_rx_queue(i as u16, clients, cl_rx_buf);
    }
}

/// Function called by the master lcore of the DPDK process.
fn do_packet_forwarding(
    ports: &mut PortInfo,
    clients: &mut Vec<Client>,
    cl_rx_buf: &mut Vec<ClientRxBuf>,
    num_clients: usize,
) {
    let mut port_num = 0_u32;
    println!("There it is!"); // DEBUG
    loop {
        // will hold the data once read
        let mut buf: Vec<*mut rte_mbuf> = Vec::with_capacity(PACKET_READ_SIZE as usize);
        // let mut buf = Mbuf::alloc_bulk(PACKET_READ_SIZE as usize).unwrap();
        let rx_count: u16;
        println!("Vars set"); // DEBUG

        /* read a port */
        rx_count = unsafe {
            _rte_eth_rx_burst(
                ports.get_port_id_by_index(port_num as usize),
                0_u16,
                // pkts.as_mut_ptr(),
                buf.as_mut_ptr(),
                PACKET_READ_SIZE.into(),
            )
        };
        println!("read port"); // DEBUG
                               // REVIEW: wrap it into Mbufs - how expensive is this operation!!!
                               // let mut pkts: Vec<Mbuf> = Vec::with_capacity(PACKET_READ_SIZE as usize);
                               // According to Rust community (Discord), the following op is linear unless from_ptr goes weird and heap allocations are required. Ballpark estimate is about ~150ns.
        let mut pkts = unsafe {
            buf.drain(..)
                .as_slice()
                .iter()
                .map(|mbuf| Mbuf::from_ptr(*mbuf))
                .collect::<Vec<_>>()
        };
        ports
            .get_rx_stats()
            .set_rx(port_num as usize, rx_count.into());

        /* Now process the NIC packets read */
        process_pkts(port_num, pkts, rx_count, clients, cl_rx_buf, num_clients);

        /* move to next port */
        port_num += 1;
        if port_num == ports.get_num_ports() {
            port_num = 0;
        }
    }
}

// fn signal_handler(signal: i32) {
//     ctrlc::set_handler(move || {
//         unsafe { stop_and_close_ports() };
//     });
// }
