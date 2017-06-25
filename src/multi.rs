extern crate comm;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate num;

use num::bigint::ToBigUint;
use std::env;
use std::thread;
use comm::address::{Address, LENGTH};
use comm::client;
use comm::network;
use comm::node;

pub fn start_multiple(host: &str, port_start: u16, port_end: u16, router_host: Option<&str>, rampup: u64) {
    let min = 0.to_biguint().unwrap();
    let max = num::pow(2.to_biguint().unwrap(), LENGTH);

    info!("Starting nodes {}:{}..{}", host, port_start, port_end);

    for port in port_start..port_end {
        debug!("-> starting {}", port);

        let routers: Vec<Box<node::Node>> = match router_host {
            Some(host) => {
                let router_node = Box::new(node::UdpNode::new(Address::null(), host));
                vec![router_node]
            }
            None => vec![]
        };

        let address = Address::random(&min, &max);
        let socket_address = (host, port);
        let network = network::Network::new(address, socket_address, routers);
        let client = client::Client::new(address);
        client.run(network);
        thread::sleep(std::time::Duration::from_millis(rampup));
    }

    info!("All running :)");
    loop { thread::park(); }
}

fn main() {
    env_logger::init().unwrap();
    let args: Vec<String> = env::args().collect();

    let host = args[1].as_str();
    let port_start = args[2].clone().parse::<u16>().unwrap();
    let port_end = args[3].clone().parse::<u16>().unwrap();
    let rampup = args[4].clone().parse::<u64>().unwrap();
    let router = args.get(5).map(|h| h.as_str());

    start_multiple(host, port_start, port_end, router, rampup);
}
