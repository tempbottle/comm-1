use address::{Addressable, Address};
use messages::outgoing;
use mio;
use node::Node;
use routing_table::{InsertOutcome, InsertionResult, RoutingTable};
use servers::Server;
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::sync::mpsc;
use std::thread;
use transaction::{TransactionId, TransactionIdGenerator};

#[derive(Clone, Debug)]
pub enum Event {
    ReceivedPacket(Address, Vec<u8>),
    Shutdown,
    Started
}

pub enum OneshotTask {
    Incoming(Vec<u8>),
    StartBootstrap,
    SendPacket(Address, Vec<u8>),
    Shutdown
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
    servers: HashMap<mio::Token, Server>,
    routing_table: RoutingTable,
    self_node: Node,
    transaction_ids: TransactionIdGenerator,
    status: Status,
    pending_actions: HashMap<TransactionId, TableAction>,
    event_listeners: Vec<mpsc::Sender<Event>>
}

impl Network {
    pub fn new(self_address: Address, servers: Vec<Server>, routers: Vec<Node>) -> Network {
        let mut transports = HashSet::new();
        let mut server_hash = HashMap::new();
        for (i, server) in servers.into_iter().enumerate() {
            let token = mio::Token(i);
            transports.insert(server.transport());
            server_hash.insert(token, server);
        }

        let self_node = Node::new(self_address, transports);
        let routing_table = RoutingTable::new(8, self_address, routers);

        Network {
            servers: server_hash,
            routing_table: routing_table,
            self_node: self_node,
            transaction_ids: TransactionIdGenerator::new(),
            status: Status::Idle,
            pending_actions: HashMap::new(),
            event_listeners: vec![]
        }
    }

    pub fn run(mut self) -> TaskSender {
        let mut event_loop_config = mio::EventLoopConfig::new();
        event_loop_config.notify_capacity(16384);
        let mut event_loop = mio::EventLoop::configured(event_loop_config).unwrap();

        for (token, server) in &mut self.servers.iter_mut() {
            let evented = server.run();
            event_loop
                .register(evented, *token, mio::EventSet::readable(), mio::PollOpt::edge())
                .expect("Couldn't register server to EventLoop");
        }

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

    fn read_server(&self, token: mio::Token, event_loop: &mut mio::EventLoop<Handler>) {
        self.servers[&token].read(event_loop.channel());
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
                        debug!("Received FindNode query for {} from {:?}", &target, &origin);
                        let response: Vec<u8> = outgoing::create_find_node_response(
                                transaction_id,
                                &self.self_node,
                                self.routing_table.nearest_live_nodes_to(&target, false));
                        origin.send(response);
                    },
                    Query::Packet(payload) => {
                        // Logging packets would be too chatty
                        self.broadcast_event(Event::ReceivedPacket(origin_address, payload));
                        let response = outgoing::create_packet_response(
                            transaction_id, &self.self_node);
                        origin.send(response);
                    },
                    Query::Ping => {
                        debug!("Received Ping from {:?}", &origin);
                        let response = outgoing::create_ping_response(
                            transaction_id,
                            &self.self_node);
                        origin.send(response);
                    }
                }

                // Always insert the origin node
                self.insert_node(origin).unwrap();
                if let Some(origin) = self.routing_table.find_node(&origin_address) {
                    origin.received_query(transaction_id);
                }
            }

            Message::Response(transaction_id, origin, response) => {
                let origin_address = origin.address();
                // Always insert the origin node
                let mut encounted_new_node = self.insert_node(origin) == Ok(InsertOutcome::Inserted);

                match response {
                    Response::FindNode(mut nodes) => {
                        for node in nodes.drain(..) {
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
                                    // TODO: Extract finish_bootstrap()
                                    self.continue_health_check(event_loop);
                                    self.continue_refresh(event_loop);
                                    match self.status {
                                        Status::Bootstrapping => {
                                            self.status = Status::Idle;
                                        }
                                        _ => { }
                                    }
                                }
                            }
                            Some(TableAction::HealthCheck(_)) => {
                                // Does not clear timeout. Lets Healthcheck proceed at regular
                                // interval.
                            },
                            Some(TableAction::RefreshBucket(_)) => {
                            }
                            None => { }
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
        self.broadcast_event(Event::Started);
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
            node.sent_query(transaction_id);
            node.send(query.clone());
        }
        transaction_id
    }

    // TODO: I should de-couple operations and transactions. Some operations, e.g. health_check
    // can be comprised of multiple transactions that must be completed before the operation is
    // complete. In this case, I should receive a ping from each questionable node before
    // health_check is done.
    fn health_check(&mut self) -> TransactionId {
        let transaction_id = self.transaction_ids.generate();

        // TODO: this should be a separate keep-alive task, but it will be
        // dependant on the type of connection we're keeping alive.
        //
        // Ping nearest node every time.
        if let Some(nearest_node) = self.routing_table.nearest().first_mut() {
            let query = outgoing::create_ping_query(
                transaction_id, &self.self_node);
            nearest_node.sent_query(transaction_id);
            nearest_node.send(query);
        }

        for node in self.routing_table.questionable_nodes().iter_mut() {
            let query = outgoing::create_ping_query(
                transaction_id, &self.self_node);
            node.sent_query(transaction_id);
            node.send(query);
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

    fn insert_node(&mut self, node: Node) -> InsertionResult {
        self.routing_table.insert(node, &self.self_node, &mut self.transaction_ids)
    }

    fn send_packet(&mut self, recipient: Address, payload: Vec<u8>, _event_loop: &mut mio::EventLoop<Handler>) {
        for node in self.routing_table.nearest_live_nodes_to(&recipient, false) {
            let transaction_id = self.transaction_ids.generate();
            let query = outgoing::create_packet_query(
                transaction_id, &self.self_node, payload.clone());
            node.sent_query(transaction_id);
            node.send(query.clone());
        }
    }

    fn shutdown(&mut self, event_loop: &mut mio::EventLoop<Handler>) {
        event_loop.shutdown();
        self.broadcast_event(Event::Shutdown);
    }

    fn broadcast_event(&self, event: Event) {
        for listener in &self.event_listeners {
            listener.send(event.clone()).expect("Failed to broadcast event");
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

    fn ready(&mut self, event_loop: &mut mio::EventLoop<Handler>, token: mio::Token, _: mio::EventSet) {
        self.network.read_server(token, event_loop);
    }

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Handler>, task: OneshotTask) {
        match task {
            OneshotTask::Incoming(data) => self.network.handle_incoming(data, event_loop),
            OneshotTask::StartBootstrap => self.network.start_bootstrap(event_loop),
            OneshotTask::SendPacket(recipient, payload) =>
                self.network.send_packet(recipient, payload, event_loop),
            OneshotTask::Shutdown => self.network.shutdown(event_loop)
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
