/*
 * Created on Mon Oct 05 2020:23:57:15
 * Created by Ratnadeep Bhattacharya
 */

pub mod constants;

use capsule::dpdk::{eal_cleanup, eal_init, Mbuf, Mempool, Memzone, SocketId};
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
            println!("Completed array mempool creation");
            nfs_mempool = nfs;
        }
        Err(e) => {
            eprintln!("NFS mempool creation failed: {}", e);
        }
    };

    /* set up ports info */
    // default cache size is 32s
    let ports_mempool: Mempool;
    match Mempool::new(
        MZ_PORT_INFO.to_string(),
        mem::size_of::<Mbuf>(),
        32 as usize,
        socket_id,
    ) {
        Ok(ports) => {
            println!("Completed ports mempool creation");
            ports_mempool = ports;
        }
        Err(e) => {
            eprintln!("Ports mempool creation failed: {}", e);
        }
    };

    /* Attempting to create a Memzone */
    let mz: Memzone;
    match Memzone::new(MZ_PORT_INFO.to_string(), mem::size_of::<Mbuf>(), socket_id) {
        Ok(m) => {
            println!("Completed memzone creation");
            mz = m;
        }
        Err(e) => {
            eprintln!("Memzone creation failed: {}", e);
        }
    };
}
