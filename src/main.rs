extern crate crypto;
extern crate mio;
extern crate num;
extern crate protobuf;
extern crate rand;
extern crate rustc_serialize;
extern crate time;

use address::Address;
use node::UdpNode;
use std::env;

mod address;
mod messages;
mod network;
mod node;
mod node_bucket;
mod routing_table;
mod transaction;
#[cfg(test)]
mod tests;

fn main() {
    use num::bigint::ToBigUint;
    use address::LENGTH;
    use std::thread;

    let args: Vec<String> = env::args().collect();
    let secret = args[1].clone();
    if secret == "multi" {
        let min = 0.to_biguint().unwrap();
        let max = num::pow(2.to_biguint().unwrap(), LENGTH);
        let port_start = args[2].clone().parse::<u16>().unwrap();
        let port_end = args[3].clone().parse::<u16>().unwrap();

        println!("Starting nodes {}..{}", port_start, port_end);

        for port in port_start..port_end {
            let address = Address::random(&min, &max);
            println!("-> {}", port);
            let routers: Vec<Box<node::Node>> = match args.get(4) {
                Some(router_port) => {
                    let router_address = ("127.0.0.1", router_port.clone().parse::<u16>().unwrap());
                    let router_node = Box::new(UdpNode::new(Address::null(), router_address));
                    vec![router_node]
                }
                None => vec![]
            };

            let socket_address = ("127.0.0.1", port);
            let self_node = node::UdpNode::new(address, socket_address);
            let network = network::Network::new(self_node, port, routers);
            thread::spawn(move|| {
                network.run();
            });
            thread::sleep(std::time::Duration::from_millis(200));
        }

        println!("All running :)");
        loop { thread::park(); }

    } else {
        let address = Address::for_content(secret.as_str());
        let port = args[2].clone().parse::<u16>().unwrap();
        let self_node = node::UdpNode::new(address, ("127.0.0.1", port));

        let routers: Vec<Box<node::Node>> = match args.get(3) {
            Some(bootstrap_port) => {
                let bootstrap_address = ("127.0.0.1", bootstrap_port.clone().parse::<u16>().unwrap());
                let bootstrap_node = Box::new(UdpNode::new(Address::null(), bootstrap_address));
                vec![bootstrap_node]
            }
            None => vec![]
        };

        let network = network::Network::new(self_node, port, routers);
        network.run();
    }
}
