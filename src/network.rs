use address::Address;
use messages;
use node::{Node, Deserialize, Serialize};
use node;
use protobuf;
use routing_table::RoutingTable;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::collections::HashSet;

pub struct Handler {
    routing_table: Arc<Mutex<RoutingTable>>,
    self_node: Arc<Box<node::Node + 'static>>,
}

impl Handler {
    pub fn new<N: node::Node + 'static>(
        routing_table: Arc<Mutex<RoutingTable>>,
        self_node: N) -> Handler {

        Handler {
            routing_table: routing_table,
            self_node: Arc::new(Box::new(self_node)),
    }

    pub fn run<N: node::Node + 'static>(
        mut self,
        messages: mpsc::Receiver<(usize, [u8; 4096])>,
        bootstrap_node: N) {

        let bootstrap_node = Box::new(bootstrap_node);

        thread::spawn(move || {
            self.query_node_for_self(&bootstrap_node);
            loop {
                let (size, buf) = messages.recv().unwrap();
                let message = protobuf::parse_from_bytes::<messages::Envelope>(&buf[0..size]).unwrap();
                self.handle_message(message);
            }
        });
    }

    fn handle_message(&mut self, message: messages::Envelope) {
        println!("Message: {:?}", message);

        match message.get_message_type() {
            messages::Envelope_Type::FIND_NODE_QUERY => {
                let mut routing_table = self.routing_table.lock().unwrap();
                let find_node_query = message.get_find_node_query();
                let origin = node::UdpNode::deserialize(find_node_query.get_origin());
                let target = Address::from_str(find_node_query.get_target());

                {
                    let nodes: Vec<&Box<node::Node>> = routing_table.nearest_to(&target);
                    println!("Nearest nodes: {:?}", nodes);
                    let nodes: Vec<messages::Node> = nodes.iter().map(|n| n.serialize()).collect();
                    let mut find_node_response = messages::FindNodeResponse::new();
                    find_node_response.set_origin(self.self_node.serialize());
                    find_node_response.set_nodes(protobuf::RepeatedField::from_slice(nodes.as_slice()));
                    let mut response = messages::Envelope::new();
                    response.set_message_type(messages::Envelope_Type::FIND_NODE_RESPONSE);
                    response.set_find_node_response(find_node_response);

                    origin.send(response);
                }

                routing_table.insert(origin);
            }
            messages::Envelope_Type::FIND_NODE_RESPONSE => {
                let mut routing_table = self.routing_table.lock().unwrap();

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
    fn query_node_for_self<N: node::Node + 'static>(&mut self, node: &Box<N>) {
        let mut find_node_query = messages::FindNodeQuery::new();
        find_node_query.set_origin(self.self_node.serialize());
        find_node_query.set_target(self.self_node.get_address().to_str());
        let mut envelope = messages::Envelope::new();
        envelope.set_message_type(messages::Envelope_Type::FIND_NODE_QUERY);
        envelope.set_find_node_query(find_node_query);
        node.send(envelope);
    }
}
