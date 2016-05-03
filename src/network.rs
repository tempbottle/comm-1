use messages::outgoing;
use mio;
use node::Node;
use node;
use routing_table::RoutingTable;
use routing_table;
use std::io::Cursor;
use std::thread;
use transaction::TransactionIdGenerator;

pub enum ScheduledTask {
    RefreshBucket(u16)
}

pub enum OneshotTask {
    Incoming(Vec<u8>),
    StartBootstrap
}

enum Status {
    Bootstrapped,
    Bootstrappping,
    Idle
}

impl Status {
    fn is_bootstrapping(&self) -> bool {
        match self {
            &Status::Bootstrappping => true,
            _ => false
        }
    }
}

pub struct Handler {
    port: u16,
    routing_table: RoutingTable,
    self_node: Box<node::Node + 'static>,
    transaction_ids: TransactionIdGenerator,
    status: Status
}

impl Handler {
    pub fn new<N: node::Node + 'static>(self_node: N, port: u16, routers: Vec<Box<Node>>) -> Handler {
        let address = self_node.get_address();
        let routing_table = routing_table::RoutingTable::new(8, address, routers);

        Handler {
            port: port,
            routing_table: routing_table,
            self_node: Box::new(self_node),
            transaction_ids: TransactionIdGenerator::new()
        }
    }

    pub fn run(mut self) {
        let mut event_loop = mio::EventLoop::new().unwrap();
        let loop_channel = event_loop.channel();

        create_incoming_udp_channel(self.port, loop_channel.clone());
        loop_channel.send(OneshotTask::StartBootstrap).unwrap();
        event_loop.run(&mut self).unwrap();
    }

    fn handle_incoming(&mut self, data: Vec<u8>) {
        use messages::incoming;
        use messages::incoming::*;
        let mut data = Cursor::new(data);
        let message = incoming::parse_from_reader(&mut data).unwrap();
        println!("Message: {:?}", message);

        match message {
            Message::Query(transaction_id, query) => {
                match query {
                    Query::FindNode(origin, target) => {
                        {
                            let nodes: Vec<&Box<node::Node>> = self.routing_table.nearest_to(&target);
                            let response = outgoing::create_find_node_response(
                                transaction_id,
                                &self.self_node,
                                nodes);
                            origin.send(response);
                        }

                        self.routing_table.insert(origin);
                    }
                }
            }
            Message::Response(_transaction_id, response) => {
                match response {
                    Response::FindNode(origin, nodes) => {
                        self.routing_table.insert(origin);

                        for node in nodes {
                            self.routing_table.insert(node);
                        }
                    }
                }
            }
        }
    }

    fn handle_start_bootstrap(&mut self, event_loop: &mut mio::EventLoop<Self>) {
        self.status = Status::Bootstrappping;
        self.find_self(event_loop);
    }

    fn find_self(&mut self, event_loop: &mut mio::EventLoop<Handler>) {
        let query = outgoing::create_find_node_query(
            self.transaction_ids.generate(),
            &self.self_node,
            self.self_node.get_address());
        if let Some(node) = self.routing_table.nearest_to(&self.self_node.get_address()).get(0) {
            node.send(query);
        }
    }
}

impl mio::Handler for Handler {
    type Timeout = ();
    type Message = OneshotTask;

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Handler>, task: OneshotTask) {
        match task {
            OneshotTask::Incoming(data) => self.handle_incoming(data),
            OneshotTask::StartBootstrap => {
                self.handle_start_bootstrap(event_loop);
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
