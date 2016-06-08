use address::{Address, LENGTH};
use client;
use network;
use node;
use num::bigint::ToBigUint;
use num;
use std::thread;
use std;

pub fn start_multiple(host: &str, port_start: u16, port_end: u16, router_host: Option<&str>) {
    let min = 0.to_biguint().unwrap();
    let max = num::pow(2.to_biguint().unwrap(), LENGTH);

    info!("Starting nodes {}..{}", port_start, port_end);

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
        let self_node = node::UdpNode::new(address, socket_address);
        let network = network::Network::new(self_node, socket_address, routers);
        let client = client::Client::new(address);
        thread::spawn(move || client.run(network, true));
        thread::sleep(std::time::Duration::from_millis(200));
    }

    info!("All running :)");
    loop { thread::park(); }
}
