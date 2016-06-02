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
mod client;
mod messages;
mod network;
mod node;
mod node_bucket;
mod multi;
mod routing_table;
mod transaction;
#[cfg(test)]
mod tests;

fn main() {
    let args: Vec<String> = env::args().collect();
    let secret = args[1].clone();

    if secret == "multi" {
        let port_start = args[2].clone().parse::<u16>().unwrap();
        let port_end = args[3].clone().parse::<u16>().unwrap();
        let router = args.get(4).map(|p| p.clone().parse::<u16>().unwrap());
        multi::start_multiple(port_start, port_end, router);
    } else {
        let address = Address::for_content(secret.as_str());
        let port = args[2].clone().parse::<u16>().unwrap();
        let self_node = node::UdpNode::new(address, ("127.0.0.1", port));

        let routers: Vec<Box<node::Node>> = match args.get(3) {
            Some(router_port) => {
                let router_address = ("127.0.0.1", router_port.clone().parse::<u16>().unwrap());
                let router_node = Box::new(UdpNode::new(Address::null(), router_address));
                vec![router_node]
            }
            None => vec![]
        };

        let network = network::Network::new(self_node, port, routers);
        let client = client::Client::new(address);
        client.run(network)
    }
}
