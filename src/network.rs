use messages::outgoing;
use mio;
use node::Node;
use node;
use routing_table::{InsertOutcome, RoutingTable};
use routing_table;
use std::collections::HashMap;
use std::io::Cursor;
use std::thread;
use transaction::{TransactionId, TransactionIdGenerator};

pub enum ScheduledTask {
    ContinueBootstrap
}

pub enum OneshotTask {
    Incoming(Vec<u8>),
    StartBootstrap
}

enum TableAction {
    Bootstrap(mio::Timeout)
}

enum Status {
    Bootstrapped,
    Bootstrapping,
    Idle
}

pub struct Handler {
    port: u16,
    routing_table: RoutingTable,
    self_node: Box<node::Node + 'static>,
    transaction_ids: TransactionIdGenerator,
    status: Status,
    pending_actions: HashMap<TransactionId, TableAction>
}

impl Handler {
    pub fn new<N: node::Node + 'static>(self_node: N, port: u16, routers: Vec<Box<Node>>) -> Handler {
        let self_address = self_node.get_address();
        let routing_table = routing_table::RoutingTable::new(8, self_address, routers);

        Handler {
            port: port,
            routing_table: routing_table,
            self_node: Box::new(self_node),
            transaction_ids: TransactionIdGenerator::new(),
            status: Status::Idle,
            pending_actions: HashMap::new()
        }
    }

    pub fn run(mut self) {
        let mut event_loop = mio::EventLoop::new().unwrap();
        let loop_channel = event_loop.channel();

        create_incoming_udp_channel(self.port, loop_channel.clone());
        loop_channel.send(OneshotTask::StartBootstrap).unwrap();
        event_loop.run(&mut self).unwrap();
    }

    fn handle_incoming(&mut self, data: Vec<u8>, event_loop: &mut mio::EventLoop<Handler>) {
        use messages::incoming;
        use messages::incoming::*;
        let mut data = Cursor::new(data);
        let message = incoming::parse_from_reader(&mut data).unwrap();
        println!("Routing table: {:?}", self.routing_table);

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

                        self.routing_table.insert(origin).unwrap();
                    }
                }
            }
            Message::Response(transaction_id, response) => {
                match response {
                    Response::FindNode(origin, mut nodes) => {
                        let mut encounted_new_node = false;
                        let mut tail = vec![origin];
                        for node in nodes.drain(..).chain(tail.drain(..)) {
                            match self.routing_table.insert(node) {
                                Ok(InsertOutcome::Inserted) => { encounted_new_node = true; }
                                _ => { }
                            }
                        }

                        match self.pending_actions.remove(&transaction_id) {
                            Some(TableAction::Bootstrap(timeout)) => {
                                event_loop.clear_timeout(timeout);
                            }
                            None => { }
                        }

                        match self.status {
                            Status::Bootstrapping => {
                                if encounted_new_node {
                                    // Continue botstrapping
                                    self.continue_bootstrap(event_loop);
                                } else {
                                    self.status = Status::Bootstrapped;
                                }
                            }
                            _ => { }
                        }
                    }
                }
            }
        }
    }

    fn start_bootstrap(&mut self, event_loop: &mut mio::EventLoop<Self>) {
        self.status = Status::Bootstrapping;
        self.continue_bootstrap(event_loop);
    }

    fn continue_bootstrap(&mut self, event_loop: &mut mio::EventLoop<Self>) {
        let transaction_id = self.find_self();
        let timeout = event_loop.timeout_ms(ScheduledTask::ContinueBootstrap, 1000).unwrap();
        self.pending_actions.insert(transaction_id, TableAction::Bootstrap(timeout));
    }

    fn find_self(&mut self) -> TransactionId {
        let transaction_id = self.transaction_ids.generate();
        let query = outgoing::create_find_node_query(
            transaction_id,
            &self.self_node,
            self.self_node.get_address());
        if let Some(node) = self.routing_table.nearest().get(0) {
            node.send(query);
        }
        transaction_id
    }
}

impl mio::Handler for Handler {
    type Timeout = ScheduledTask;
    type Message = OneshotTask;

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Handler>, task: OneshotTask) {
        match task {
            OneshotTask::Incoming(data) => self.handle_incoming(data, event_loop),
            OneshotTask::StartBootstrap => self.start_bootstrap(event_loop)
        }
    }

    fn timeout(&mut self, event_loop: &mut mio::EventLoop<Handler>, timeout: ScheduledTask) {
        match timeout {
            ScheduledTask::ContinueBootstrap => self.continue_bootstrap(event_loop)
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
