extern crate crypto;
extern crate mio;
extern crate num;
extern crate protobuf;
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let secret = args[1].clone();
    let address = Address::for_content(secret.as_str());
    let port = args[2].clone().parse::<u16>().unwrap();
    let self_node = node::UdpNode::new(address, ("127.0.0.1", port));

    let bootstrap_address = ("127.0.0.1", args[3].clone().parse::<u16>().unwrap());
    let bootstrap_node = Box::new(UdpNode::new(Address::null(), bootstrap_address));

    let handler = network::Handler::new(self_node, port, vec![bootstrap_node]);
    handler.run();
}
