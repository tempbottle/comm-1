use address::Address;
use messages::outgoing;
use mio;
use node::Node;
use node;
use routing_table::{InsertOutcome, InsertionResult, RoutingTable};
use routing_table;
use std::collections::HashMap;
use std::io::Cursor;
use std::thread;
use transaction::{TransactionId, TransactionIdGenerator};

pub enum ScheduledTask {
    ContinueBootstrap,
    ContinueHealthCheck,
    ContinueRefresh
}

pub enum OneshotTask {
    Incoming(Vec<u8>),
    StartBootstrap
}

enum TableAction {
    Bootstrap(mio::Timeout),
    HealthCheck(mio::Timeout),
    RefreshBucket(mio::Timeout)
}

enum Status {
    Bootstrapping,
    Idle
}

pub struct Network {
    port: u16,
    routing_table: RoutingTable,
    self_node: Box<node::Node + 'static>,
    transaction_ids: TransactionIdGenerator,
    status: Status,
    pending_actions: HashMap<TransactionId, TableAction>
}

impl Network {
    pub fn new<N: node::Node + 'static>(self_node: N, port: u16, routers: Vec<Box<Node>>) -> Network {
        let self_address = self_node.get_address().clone();
        let routing_table = routing_table::RoutingTable::new(8, self_address, routers);

        Network {
            port: port,
            routing_table: routing_table,
            self_node: Box::new(self_node),
            transaction_ids: TransactionIdGenerator::new(),
            status: Status::Idle,
            pending_actions: HashMap::new()
        }
    }

    pub fn run(self) {
        let mut event_loop_config = mio::EventLoopConfig::new();
        event_loop_config.notify_capacity(16384);
        let mut event_loop = mio::EventLoop::configured(event_loop_config).unwrap();
        let loop_channel = event_loop.channel();

        create_incoming_udp_channel(self.port, loop_channel.clone());
        loop_channel.send(OneshotTask::StartBootstrap).unwrap();
        let mut handler = Handler::new(self);
        event_loop.run(&mut handler).unwrap();
    }

    fn handle_incoming(&mut self, data: Vec<u8>, event_loop: &mut mio::EventLoop<Handler>) {
        use messages::incoming;
        use messages::incoming::*;
        let mut data = Cursor::new(data);
        let message = incoming::parse_from_reader(&mut data).unwrap();

        match message {
            Message::Query(transaction_id, origin, query) => {
                let origin_address = origin.get_address();
                match query {
                    Query::FindNode(target) => {
                        let response: Vec<u8> = outgoing::create_find_node_response(
                                transaction_id,
                                &self.self_node,
                                self.routing_table.nearest_to(&target, false));
                        origin.send(response);

                        self.insert_node(origin).unwrap();
                        {
                            if let Some(origin) = self.routing_table.find_node(&origin_address) {
                                origin.received_query(transaction_id);
                            }
                        }
                    },
                    Query::Ping => {
                        let response = outgoing::create_ping_response(
                            transaction_id,
                            &self.self_node);
                        origin.send(response);

                        self.insert_node(origin).unwrap();
                        if let Some(origin) = self.routing_table.find_node(&origin_address) {
                            origin.received_query(transaction_id);
                        }
                    }
                }
            }
            Message::Response(transaction_id, origin, response) => {
                let origin_address = origin.get_address();
                match response {
                    Response::FindNode(mut nodes) => {
                        let mut encounted_new_node = false;
                        let mut tail = vec![origin];
                        for node in nodes.drain(..).chain(tail.drain(..)) {
                            match self.insert_node(node) {
                                Ok(InsertOutcome::Inserted) => {
                                    encounted_new_node = true;
                                }
                                Err(error) => { panic!(error) }
                                _ => { }
                            }
                        }

                        {
                            let mut origin = self.routing_table
                                .find_node(&origin_address)
                                .expect("Got find node response from unknown node");
                            origin.received_response(transaction_id);
                        }

                        match self.pending_actions.remove(&transaction_id) {
                            Some(TableAction::Bootstrap(timeout)) => {
                                event_loop.clear_timeout(timeout);
                                if encounted_new_node {
                                    self.continue_bootstrap(event_loop);
                                } else {
                                    self.continue_health_check(event_loop);
                                    self.continue_refresh(event_loop)
                                }
                            }
                            Some(TableAction::HealthCheck(_)) => {
                            },
                            Some(TableAction::RefreshBucket(_)) => {
                            }
                            None => { }
                        }

                        match self.status {
                            Status::Bootstrapping => {
                                self.status = Status::Idle;
                            }
                            _ => { }
                        }
                    }
                    Response::Ping => {
                        let mut origin = self.routing_table.find_node(&origin_address).expect("Got ping response from unknown node");
                        origin.received_response(transaction_id);
                    }
                }
            }
        }
    }

    fn start_bootstrap(&mut self, event_loop: &mut mio::EventLoop<Handler>) {
        self.status = Status::Bootstrapping;
        self.continue_bootstrap(event_loop);
    }

    fn continue_bootstrap(&mut self, event_loop: &mut mio::EventLoop<Handler>) {
        let address = &self.self_node.get_address();
        let transaction_id = self.find_node(address);
        let timeout = event_loop.timeout_ms(ScheduledTask::ContinueBootstrap, 1000).unwrap();
        self.pending_actions.insert(transaction_id, TableAction::Bootstrap(timeout));
    }

    fn continue_health_check(&mut self, event_loop: &mut mio::EventLoop<Handler>) {
        let transaction_id = self.health_check();
        let timeout = event_loop.timeout_ms(ScheduledTask::ContinueHealthCheck, 1000).unwrap();
        self.pending_actions.insert(transaction_id, TableAction::HealthCheck(timeout));
    }

    fn continue_refresh(&mut self, event_loop: &mut mio::EventLoop<Handler>) {
        let timeout = event_loop.timeout_ms(ScheduledTask::ContinueRefresh, 1000).unwrap();

        match self.refresh_bucket() {
            Some(transaction_id) => {
                self.pending_actions.insert(transaction_id, TableAction::RefreshBucket(timeout));
            }
            None => { }
        }
    }

    fn find_node(&mut self, address: &Address) -> TransactionId {
        let transaction_id = self.transaction_ids.generate();
        let query = outgoing::create_find_node_query(
            transaction_id,
            &self.self_node,
            address);
        for node in self.routing_table.nearest() {
            {
                node.send(query.clone());
            }
            node.sent_query(transaction_id);
        }
        transaction_id
    }

    fn health_check(&mut self) -> TransactionId {
        let transaction_id = self.transaction_ids.generate();
        if let Some(node) = self.routing_table.questionable_nodes().get_mut(0) {
            let query = outgoing::create_ping_query(
                transaction_id, &self.self_node);
            node.send(query);
            node.sent_query(transaction_id);
        }
        transaction_id
    }

    fn refresh_bucket(&mut self) -> Option<TransactionId> {
        if let Some(address) = self.address_to_find_for_refresh() {
            Some(self.find_node(&address))
        } else {
            None
        }
    }

    fn address_to_find_for_refresh(&self) -> Option<Address> {
        match self.routing_table.bucket_needing_refresh() {
            Some(bucket) => {
                println!("refreshing {:?}", bucket);
                Some(bucket.random_address_in_space())
            }
            None => None
        }
    }

    fn insert_node(&mut self, node: Box<Node>) -> InsertionResult {
        self.routing_table.insert(node, &self.self_node, &mut self.transaction_ids)
    }
}

struct Handler {
    network: Network
}

impl Handler {
    fn new(network: Network) -> Handler {
        Handler {
            network: network
        }
    }
}

impl mio::Handler for Handler {
    type Timeout = ScheduledTask;
    type Message = OneshotTask;

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Handler>, task: OneshotTask) {
        match task {
            OneshotTask::Incoming(data) => self.network.handle_incoming(data, event_loop),
            OneshotTask::StartBootstrap => self.network.start_bootstrap(event_loop)
        }
    }

    fn timeout(&mut self, event_loop: &mut mio::EventLoop<Handler>, timeout: ScheduledTask) {
        match timeout {
            ScheduledTask::ContinueBootstrap => self.network.continue_bootstrap(event_loop),
            ScheduledTask::ContinueHealthCheck => self.network.continue_health_check(event_loop),
            ScheduledTask::ContinueRefresh => self.network.continue_refresh(event_loop)
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
