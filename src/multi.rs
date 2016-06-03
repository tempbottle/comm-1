use address::{Address, LENGTH};
use network;
use node;
use num::bigint::ToBigUint;
use num;
use std::thread;
use std;

pub fn start_multiple(port_start: u16, port_end: u16, router_host: Option<&str>) {
    let min = 0.to_biguint().unwrap();
    let max = num::pow(2.to_biguint().unwrap(), LENGTH);

    println!("Starting nodes {}..{}", port_start, port_end);

    for port in port_start..port_end {
        println!("-> {}", port);

        let routers: Vec<Box<node::Node>> = match router_host {
            Some(host) => {
                let router_node = Box::new(node::UdpNode::new(Address::null(), host));
                vec![router_node]
            }
            None => vec![]
        };

        let address = Address::random(&min, &max);
        let socket_address = ("127.0.0.1", port);
        let self_node = node::UdpNode::new(address, socket_address);
        let network = network::Network::new(self_node, socket_address, routers);
        network.run();
        thread::sleep(std::time::Duration::from_millis(200));
    }

    println!("All running :)");
    loop { thread::park(); }
}
