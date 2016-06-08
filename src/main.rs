extern crate env_logger;
extern crate crypto;
#[macro_use]
extern crate log;
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
    env_logger::init().unwrap();
    let args: Vec<String> = env::args().collect();
    let secret = args[1].clone();

    if secret == "multi" {
        let host = args[2].as_str();
        let port_start = args[3].clone().parse::<u16>().unwrap();
        let port_end = args[4].clone().parse::<u16>().unwrap();
        let router = args.get(5).map(|h| h.as_str());
        multi::start_multiple(host, port_start, port_end, router);
    } else {
        let address = Address::for_content(secret.as_str());
        let host = args[2].as_str();
        let self_node = node::UdpNode::new(address, host);

        let routers: Vec<Box<node::Node>> = match args.get(3) {
            Some(router_address) => {
                let router_node = Box::new(UdpNode::new(Address::null(), router_address.as_str()));
                vec![router_node]
            }
            None => vec![]
        };

        let network = network::Network::new(self_node, host, routers);
        let client = client::Client::new(address);
        client.run(network)
    }
}
