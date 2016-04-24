use address::Address;
use messages;
use mio;
use node::{Node, Deserialize, Serialize};
use node;
use protobuf::Message;
use protobuf;
use routing_table::RoutingTable;
use routing_table;
use std::io::Cursor;
use std::str::FromStr;
use std::thread;
use transaction::TransactionIdGenerator;

pub enum OneshotTask {
    Incoming(Vec<u8>),
    StartBootstrap(Box<node::Node>)
}

pub struct Handler {
    port: u16,
    routing_table: RoutingTable,
    self_node: Box<node::Node + 'static>,
    transaction_ids: TransactionIdGenerator
}

impl Handler {
    pub fn new<N: node::Node + 'static>(self_node: N, port: u16) -> Handler {
        let address = self_node.get_address();
        let routing_table = routing_table::RoutingTable::new(8, address);

        Handler {
            port: port,
            routing_table: routing_table,
            self_node: Box::new(self_node),
            transaction_ids: TransactionIdGenerator::new()
        }
    }

    pub fn run<N: node::Node + 'static>(mut self, bootstrap_node: N) {
        let bootstrap_node = Box::new(bootstrap_node);
        let mut event_loop = mio::EventLoop::new().unwrap();
        let loop_channel = event_loop.channel();

        create_incoming_udp_channel(self.port, loop_channel.clone());
        loop_channel.send(OneshotTask::StartBootstrap(bootstrap_node)).unwrap();
        event_loop.run(&mut self).unwrap();
    }

    fn handle_incoming(&mut self, data: Vec<u8>) {
        use messages::*;
        let mut data = Cursor::new(data);
        let message = protobuf::parse_from_reader::<protobufs::Envelope>(&mut data).unwrap();
        println!("Message: {:?}", message);

        match message.get_message_type() {
            protobufs::Envelope_Type::FIND_NODE_QUERY => {
                let find_node_query = message.get_find_node_query();
                let origin = node::UdpNode::deserialize(find_node_query.get_origin());
                let target = Address::from_str(find_node_query.get_target());

                {
                    let nodes: Vec<&Box<node::Node>> = self.routing_table.nearest_to(&target);
                    let response = Message::Response(
                        self.transaction_ids.generate(),
                        Response::FindNode(&self.self_node, nodes));
                    origin.send(response.serialize());
                }

                self.routing_table.insert(origin);
            }
            protobufs::Envelope_Type::FIND_NODE_RESPONSE => {
                let find_node_response = message.get_find_node_response();
                let origin = node::UdpNode::deserialize(find_node_response.get_origin());
                self.routing_table.insert(origin);

                for node in find_node_response.get_nodes() {
                    let node = node::UdpNode::deserialize(node);
                    self.routing_table.insert(node);
                }
            }
        }
    }

    fn query_node_for_self(&mut self, node: &Box<Node>) {
        use messages::*;
        let query = Message::Query(
            self.transaction_ids.generate(),
            Query::FindNode(&self.self_node, self.self_node.get_address()));
        node.send(query.serialize());
    }
}

impl mio::Handler for Handler {
    type Timeout = ();
    type Message = OneshotTask;

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Handler>, task: OneshotTask) {
        match task {
            OneshotTask::Incoming(data) => self.handle_incoming(data),
            OneshotTask::StartBootstrap(node) => {
                self.query_node_for_self(&node);
            }
        }
    }
}

fn create_incoming_udp_channel(port: u16, sender: mio::Sender<OneshotTask>) {
    use std::net::UdpSocket;
    thread::spawn(move || {
        let address = ("0.0.0.0", port);
        let socket = UdpSocket::bind(address).unwrap();
        loop {
            let mut buf = [0; 4096];
            match socket.recv_from(&mut buf) {
                Ok((size, _src)) => {
                    sender.send(OneshotTask::Incoming(buf[..size].iter().cloned().collect())).unwrap();
                }
                Err(e) => panic!("Error receiving from server: {}", e)
            }
        }
    });
}
