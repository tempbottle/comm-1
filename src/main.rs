extern crate crypto;
extern crate num;
extern crate protobuf;
extern crate rustc_serialize;
extern crate time;

use address::Address;
use node::UdpNode;
use server::Server;
use std::env;
use std::sync::{Arc, Mutex, mpsc};

mod address;
mod messages;
mod network;
mod node;
mod node_bucket;
mod routing_table;
mod server;
mod transaction;

fn main() {
    let args: Vec<String> = env::args().collect();
    let secret = args[1].clone();
    let address = Address::for_content(secret.as_str());
    let port = args[2].clone().parse::<u16>().unwrap();

    let (tx, incoming) = mpsc::channel();
    server::UdpServer::new(port).start(tx);

    let self_node = node::UdpNode::new(address, ("127.0.0.1", port));
    let routing_table = routing_table::RoutingTable::new(8, address);
    let bootstrap_address = ("127.0.0.1", args[3].clone().parse::<u16>().unwrap());
    let bootstrap_node = UdpNode::new(Address::null(), bootstrap_address);

    let handler = network::Handler::new(Arc::new(Mutex::new(routing_table)), self_node);

    handler.run(incoming, bootstrap_node);

    loop { }
}
