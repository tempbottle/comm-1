extern crate crypto;
extern crate num;
extern crate rustc_serialize;
extern crate time;
extern crate protobuf;

use address::Address;
use node::{Node, Deserialize, Serialize, UdpNode};
use server::Server;
use std::env;
use std::sync::mpsc;

mod address;
mod messages;
mod node;
mod node_bucket;
mod routing_table;
mod server;

fn main() {
    let args: Vec<String> = env::args().collect();
    let secret = args[1].clone();
    let address = Address::for_content(secret.as_str());
    let port = args[2].clone().parse::<u16>().unwrap();

    let (tx, messages) = mpsc::channel();
    server::UdpServer::new(port).start(tx);

    let mut my_origin = messages::Node::new();
    my_origin.set_id(address.to_str());
    my_origin.set_ip_address(vec![127, 0, 0, 1]);
    my_origin.set_port(port as u32);

    let mut routing_table = routing_table::RoutingTable::new(8, address);

    if let Some(bootstrap) = args.get(3) {
        println!("Bootstrapping");
        let mut find_node_query = messages::FindNodeQuery::new();
        find_node_query.set_origin(my_origin.clone());
        find_node_query.set_target(address.to_str());
        let mut envelope = messages::Envelope::new();
        envelope.set_message_type(messages::Envelope_Type::FIND_NODE_QUERY);
        envelope.set_find_node_query(find_node_query);
        let node = UdpNode::new(Address::null(), ("127.0.0.1", bootstrap.clone().parse::<u16>().unwrap()));
        node.send(envelope);
    }

    loop {
        let (size, buf) = messages.recv().unwrap();
        let message = protobuf::parse_from_bytes::<messages::Envelope>(&buf[0..size]).unwrap();
        println!("Message: {:?}", message);

        match message.get_message_type() {
            messages::Envelope_Type::FIND_NODE_QUERY => {
                let find_node_query = message.get_find_node_query();
                let origin = node::UdpNode::deserialize(find_node_query.get_origin());
                let target = Address::from_str(find_node_query.get_target());

                {
                    let nodes: Vec<&Box<node::Node>> = routing_table.nearest_to(&target);
                    println!("Nearest nodes: {:?}", nodes);
                    let nodes: Vec<messages::Node> = nodes.iter().map(|n| n.serialize()).collect();
                    let mut find_node_response = messages::FindNodeResponse::new();
                    find_node_response.set_origin(my_origin.clone());
                    find_node_response.set_nodes(
                        protobuf::RepeatedField::from_slice(nodes.as_slice())
                    );
                    let mut response = messages::Envelope::new();
                    response.set_message_type(messages::Envelope_Type::FIND_NODE_RESPONSE);
                    response.set_find_node_response(find_node_response);

                    origin.send(response);
                }

                routing_table.insert(origin);
            }
            messages::Envelope_Type::FIND_NODE_RESPONSE => {
                let find_node_response = message.get_find_node_response();
                let origin = node::UdpNode::deserialize(find_node_response.get_origin());
                routing_table.insert(origin);

                for node in find_node_response.get_nodes() {
                    let node = node::UdpNode::deserialize(node);
                    routing_table.insert(node);
                }
            }
        }
        println!("Routing Table: {:?}", routing_table);
    }
}
