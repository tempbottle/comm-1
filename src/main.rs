extern crate crypto;
extern crate num;
extern crate protobuf;
extern crate rustc_serialize;
extern crate time;

use address::Address;
use node::{Node, UdpNode};
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

    let mut my_origin = messages::Node::new();
    my_origin.set_id(address.to_str());
    my_origin.set_ip_address(vec![127, 0, 0, 1]);
    my_origin.set_port(port as u32);

    let routing_table = routing_table::RoutingTable::new(8, address);

    if let Some(bootstrap) = args.get(3) {
        println!("Bootstrapping");
        let bootstrap_address = ("127.0.0.1", bootstrap.clone().parse::<u16>().unwrap());
        let mut find_node_query = messages::FindNodeQuery::new();
        find_node_query.set_origin(my_origin.clone());
        find_node_query.set_target(address.to_str());
        let mut envelope = messages::Envelope::new();
        envelope.set_message_type(messages::Envelope_Type::FIND_NODE_QUERY);
        envelope.set_find_node_query(find_node_query);
        let node = UdpNode::new(Address::null(), bootstrap_address);
        node.send(envelope);
    }

    let handler = network::Handler::new(Arc::new(Mutex::new(routing_table)));
    handler.run(my_origin.clone(), incoming);
    loop { }
}
