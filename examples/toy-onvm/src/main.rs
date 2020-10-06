/*
 * Created on Mon Oct 05 2020:23:57:15
 * Created by Ratnadeep Bhattacharya
 */

pub mod constants;

use capsule::dpdk::{eal_cleanup, eal_init, Mbuf, Mempool, Memzone, Ring, SocketId};
use capsule_ffi::rte_eth_dev_count_avail;
use std::mem;

use constants::{MAX_NFS, MZ_NF_INFO, MZ_PORT_INFO};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    println!("args: {:?}", args);
    let mut all_args = args.split(|elem| elem == "--");
    let mut _s;
    let dpdk_args = match all_args.next() {
        Some(s) => s,
        None => {
            _s = [("".to_string())];
            &_s
        }
    };
    println!("dpdk_args: {:?}", dpdk_args);
    let onvm_args = match all_args.next() {
        Some(s) => s,
        None => {
            _s = [("".to_string())];
            &_s
        }
    };
    println!("onvm_args: {:?}", onvm_args);
    if let Err(_) = eal_init(args) {
        if let Err(e) = eal_cleanup() {
            panic!("{:?}", e);
        }
    };
    println!("EAL started successfully");

    /* get total number of ports */
    let total_ports = unsafe { rte_eth_dev_count_avail() };
    println!("Got total ports: {}", total_ports);

    let socket_id = SocketId::current();

    /* set up a mempool array */
    // default cache size is 32
    let nfs_mempool: Mempool;
    match Mempool::new(
        MZ_NF_INFO.to_string(),
        mem::size_of::<Mbuf>() * MAX_NFS,
        32 as usize,
        socket_id,
    ) {
        Ok(nfs) => {
            nfs_mempool = nfs;
            println!("Completed array mempool creation");
            println!("pool: {:?}", nfs_mempool);
        }
        Err(e) => {
            eprintln!("NFS mempool creation failed: {}", e);
        }
    };
    // if the error branch executed then nfs_mempool is not initialised here

    /* set up ports info */
    // default cache size is 32s
    let ports_mempool: Mempool;
    match Mempool::new(
        MZ_PORT_INFO.to_string(),
        mem::size_of::<Mbuf>() * MAX_NFS,
        32 as usize,
        socket_id,
    ) {
        Ok(ports) => {
            ports_mempool = ports;
            println!("Completed ports mempool creation");
            println!("pool: {:?}", ports_mempool);
        }
        Err(e) => {
            eprintln!("Ports mempool creation failed: {}", e);
        }
    };
    // if the error branch executed then ports_mempool is not initialised here

    /* Attempting to create a Memzone */
    let mz: Memzone;
    let MZ_MEMZONE_INFO = "TEST_MEMZONE";
    match Memzone::new(
        MZ_MEMZONE_INFO.to_string(),
        mem::size_of::<Mbuf>(),
        socket_id,
    ) {
        Ok(m) => {
            mz = m;
            println!("Completed memzone creation");
            println!("pool: {:?}", mz);
        }
        Err(e) => {
            eprintln!("Memzone creation failed: {}", e);
        }
    };
    // if the error branch executed then mz is not initialised here

    /* Attempting to create a Memzone */
    let ring: Ring;
    let MZ_RING_INFO = "TEST_RING";
    match Ring::new(MZ_RING_INFO.to_string(), mem::size_of::<Mbuf>(), socket_id) {
        Ok(r) => {
            ring = r;
            println!("Completed ring creation");
            println!("ring: {:?}", ring);
        }
        Err(e) => {
            eprintln!("Ring creation failed: {}", e);
        }
    };
    // if the error branch executed then ring is not initialised here
}
