use address::Address;
use messages;
use node::{Node, Deserialize, Serialize};
use node;
use protobuf;
use routing_table::RoutingTable;
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;
use transaction::TransactionIdGenerator;
use routing_table;
use mio::udp;
use mio;

const SERVER: mio::Token = mio::Token(0);

pub struct Handler {
    server: udp::UdpSocket,
    routing_table: RoutingTable,
    self_node: Box<node::Node + 'static>,
    transaction_ids: TransactionIdGenerator
}

impl Handler {
    pub fn new<N: node::Node + 'static>(self_node: N, port: u16) -> Handler {
        let address = self_node.get_address();
        let routing_table = routing_table::RoutingTable::new(8, address);
        let socket_address = SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), port);
        let server = udp::UdpSocket::bound(&socket_address).unwrap();

        Handler {
            server: server,
            routing_table: routing_table,
            self_node: Box::new(self_node),
            transaction_ids: TransactionIdGenerator::new()
        }
    }

    pub fn run<N: node::Node + 'static>(&mut self, bootstrap_node: N) {
        let bootstrap_node = Box::new(bootstrap_node);
        let mut event_loop = mio::EventLoop::new().unwrap();
        event_loop.register(&self.server,
                            SERVER,
                            mio::EventSet::readable(),
                            mio::PollOpt::level()).unwrap();
        self.query_node_for_self(&bootstrap_node);
        event_loop.run(self).unwrap();
    }

    fn handle_message(&mut self, message: messages::Envelope) {
        println!("Message: {:?}", message);

        match message.get_message_type() {
            messages::Envelope_Type::FIND_NODE_QUERY => {
                let find_node_query = message.get_find_node_query();
                let origin = node::UdpNode::deserialize(find_node_query.get_origin());
                let target = Address::from_str(find_node_query.get_target());

                {
                    let nodes: Vec<&Box<node::Node>> = self.routing_table.nearest_to(&target);
                    println!("Nearest nodes: {:?}", nodes);
                    let nodes: Vec<messages::Node> = nodes.iter().map(|n| n.serialize()).collect();
                    let mut find_node_response = messages::FindNodeResponse::new();
                    find_node_response.set_origin(self.self_node.serialize());
                    find_node_response.set_nodes(protobuf::RepeatedField::from_slice(nodes.as_slice()));
                    let mut response = messages::Envelope::new();
                    response.set_message_type(messages::Envelope_Type::FIND_NODE_RESPONSE);
                    response.set_find_node_response(find_node_response);
                    response.set_transaction_id(self.transaction_ids.generate());
                    let buf = response.write_to_bytes().unwrap();
                    origin.send(buf);
                }

                self.routing_table.insert(origin);
            }
            messages::Envelope_Type::FIND_NODE_RESPONSE => {
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

    fn query_node_for_self<N: node::Node + 'static>(&mut self, node: &Box<N>) {
        let transaction_id = self.transaction_ids.generate();
        let mut find_node_query = messages::FindNodeQuery::new();
        find_node_query.set_origin(self.self_node.serialize());
        find_node_query.set_target(self.self_node.get_address().to_str());
        let mut envelope = messages::Envelope::new();
        envelope.set_message_type(messages::Envelope_Type::FIND_NODE_QUERY);
        envelope.set_find_node_query(find_node_query);
        envelope.set_transaction_id(transaction_id);
        let buf = envelope.write_to_bytes().unwrap();
        node.send(buf);
    }
}

impl mio::Handler for Handler {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut mio::EventLoop<Handler>, token: mio::Token, events: mio::EventSet) {
        match token {
            SERVER => {
                assert!(events.is_readable());

                let mut buf = [0; 4096];
                match self.server.recv_from(&mut buf) {
                    Ok(Some((size, _src))) => {
                        let message = protobuf::parse_from_bytes::<messages::Envelope>(&buf[0..size]).unwrap();
                        self.handle_message(message);
                    }
                    Ok(None) => println!("Looks like the socket wasn't ready after all"),
                    Err(e) => panic!("Error receiving from server: {}", e)
                }
            }
            _ => panic!("Received a connection from some other socket?!")
        }
    }
}
