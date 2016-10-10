use address::Address;
use messages::outgoing;
use mio;
use node;
use routing_table::{InsertOutcome, InsertionResult, RoutingTable};
use std::collections::HashMap;
use std::io::Cursor;
use std::net::{UdpSocket, ToSocketAddrs, SocketAddr};
use std::sync::mpsc;
use std::thread;
use stun;
use transaction::{TransactionId, TransactionIdGenerator};

#[derive(Debug)]
pub enum Event {
    ReceivedPacket(Address, Vec<u8>)
}

pub enum OneshotTask {
    Incoming(Vec<u8>),
    StartBootstrap,
    SendPacket(Address, Vec<u8>)
}

pub enum ScheduledTask {
    ContinueBootstrap,
    ContinueHealthCheck,
    ContinueRefresh
}

enum TableAction {
    Bootstrap(mio::Timeout),
    HealthCheck(mio::Timeout),
    RefreshBucket(mio::Timeout)
}

pub type TaskSender = mio::Sender<OneshotTask>;

enum Status {
    Bootstrapping,
    Idle
}

pub struct Network {
    host: SocketAddr,
    routing_table: RoutingTable,
    self_node: Box<node::Node + 'static>,
    transaction_ids: TransactionIdGenerator,
    status: Status,
    pending_actions: HashMap<TransactionId, TableAction>,
    event_listeners: Vec<mpsc::Sender<Event>>
}

impl Network {
    pub fn new<T: ToSocketAddrs>(self_address: Address, host: T, routers: Vec<Box<node::Node>>) -> Network {
        let host = host.to_socket_addrs().unwrap().next().unwrap();
        let mapped_host = stun::get_mapped_address(host).unwrap();
        let self_node = node::UdpNode::new(self_address, mapped_host);
        let routing_table = RoutingTable::new(8, self_address, routers);

        Network {
            host: host,
            routing_table: routing_table,
            self_node: Box::new(self_node),
            transaction_ids: TransactionIdGenerator::new(),
            status: Status::Idle,
            pending_actions: HashMap::new(),
            event_listeners: vec![]
        }
    }

    pub fn run(self) -> TaskSender {
        let mut event_loop_config = mio::EventLoopConfig::new();
        event_loop_config.notify_capacity(16384);
        let mut event_loop = mio::EventLoop::configured(event_loop_config).unwrap();

        create_incoming_udp_channel(self.host, event_loop.channel());
        event_loop.channel().send(OneshotTask::StartBootstrap).unwrap();
        info!("Running server at {:?}", self.self_node);
        let mut handler = Handler::new(self);
        let task_sender = event_loop.channel();
        thread::spawn(move|| event_loop.run(&mut handler).unwrap());
        task_sender
    }

    pub fn register_event_listener(&mut self, event_listener: mpsc::Sender<Event>) {
        self.event_listeners.push(event_listener);
    }

    fn handle_incoming(&mut self, data: Vec<u8>, event_loop: &mut mio::EventLoop<Handler>) {
        use messages::incoming::{Message, Query, Response, self};
        let mut data = Cursor::new(data);
        let message = incoming::parse_from_reader(&mut data).unwrap();

        match message {
            Message::Query(transaction_id, origin, query) => {
                let origin_address = origin.address();
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
                    Query::Packet(payload) => {
                        for listener in &self.event_listeners {
                            listener.send(Event::ReceivedPacket(origin_address, payload.clone())).unwrap();
                        }
                        let response = outgoing::create_packet_response(
                            transaction_id, &self.self_node);
                        origin.send(response);
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
                let origin_address = origin.address();
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
                            if let Some(mut origin) = self.routing_table.find_node(&origin_address) {
                                origin.received_response(transaction_id);
                            }
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
                    Response::Packet => {
                        if let Some(mut origin) = self.routing_table.find_node(&origin_address) {
                            origin.received_response(transaction_id);
                        }
                    }
                    Response::Ping => {
                        if let Some(mut origin) = self.routing_table.find_node(&origin_address) {
                            origin.received_response(transaction_id);
                        }
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
        let address = &self.self_node.address();
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
            node.send(query.clone());
            node.sent_query(transaction_id);
        }
        transaction_id
    }

    fn health_check(&mut self) -> TransactionId {
        self.routing_table.remove_bad_nodes();
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
                Some(bucket.random_address_in_space())
            }
            None => None
        }
    }

    fn insert_node(&mut self, node: Box<node::Node>) -> InsertionResult {
        self.routing_table.insert(node, &self.self_node, &mut self.transaction_ids)
    }

    fn send_packet(&mut self, recipient: Address, payload: Vec<u8>, _event_loop: &mut mio::EventLoop<Handler>) {
        for node in self.routing_table.nearest_to(&recipient, false) {
            let transaction_id = self.transaction_ids.generate();
            let query = outgoing::create_packet_query(
                transaction_id, &self.self_node, payload.clone());
            node.send(query.clone());
            node.sent_query(transaction_id);
        }
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
            OneshotTask::StartBootstrap => self.network.start_bootstrap(event_loop),
            OneshotTask::SendPacket(recipient, payload) =>
                self.network.send_packet(recipient, payload, event_loop)
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

fn create_incoming_udp_channel(host: SocketAddr, sender: TaskSender) {
    thread::spawn(move || {
        let host = ("0.0.0.0", host.port());
        let socket = UdpSocket::bind(host).unwrap();
        loop {
            let mut buf = [0; 4096];
            match socket.recv_from(&mut buf) {
                Ok((size, _src)) => {
                    sender
                        .send(OneshotTask::Incoming(buf[..size].iter().cloned().collect()))
                        .unwrap_or_else(|err| info!("Couldn't handling incoming: {:?}", err));
                }
                Err(e) => panic!("Error receiving from server: {}", e)
            }
        }
    });
}
