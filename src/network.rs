use protobuf;
use messages;
use node;
use node::{Node, Deserialize, Serialize};
use address::Address;
use routing_table::RoutingTable;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

pub struct Handler {
    routing_table: Arc<Mutex<RoutingTable>>
}

impl Handler {
    pub fn new(routing_table: Arc<Mutex<RoutingTable>>) -> Handler {
        Handler { routing_table: routing_table }
    }

    pub fn run(self, my_origin: messages::Node, messages: mpsc::Receiver<(usize, [u8; 4096])>) {
        thread::spawn(move || {
            loop {
                let (size, buf) = messages.recv().unwrap();
                let message = protobuf::parse_from_bytes::<messages::Envelope>(&buf[0..size]).unwrap();
                let mut routing_table = self.routing_table.lock().unwrap();
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
            }
        });
    }
}
