use address::{Address, Addressable};
use messages;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::net::{SocketAddr, UdpSocket, IpAddr, Ipv4Addr};
use time;
use transaction::TransactionId;

/// The maximum number of queries sent without a response before a node is considered bad.
pub const FAILED_TO_RESPOND_THRESHOLD: usize = 5;

/// A node becomes questionable if it hasn't been heard from in this many minutes.
pub const MINUTES_UNTIL_QUESTIONABLE: i64 = 15;

/// Anything that needs to be serialized for transfer or storage.
///
/// TODO: this and `deserialize` should probably both be a part of a `Serializable` trait so that
/// other types can impl it instead of the `incoming` and `outgoing` functions in `messages`.
pub trait Serialize {
    fn serialize(&self) -> messages::protobufs::Node;
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Good,
    Questionable,
    Bad
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Transport {
    Udp(UdpTransport)
}

impl Transport {
    /// Creates a Transport trait object (thus it must be boxed)
    fn deserialize(message: &messages::protobufs::Transport) -> Transport {
        match message.get_transport_type() {
            messages::protobufs::Transport_Type::UDP => {
                let message = message.get_udp_transport();
                let ip = message.get_ip_address();
                let ip = Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
                let port = message.get_port() as u16;
                Transport::Udp(UdpTransport::new(SocketAddr::new(IpAddr::V4(ip), port)))
            }
        }
    }

    fn send(&self, message: Vec<u8>) {
        match self {
            Transport::Udp(transport) => transport.send(message)
        }
    }

    fn serialize(&self) -> messages::protobufs::Transport {
        match self {
            Transport::Udp(transport) => transport.serialize()
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub struct UdpTransport {
    socket_address: SocketAddr
}

impl UdpTransport {
    pub fn new(socket_address: SocketAddr) -> UdpTransport {
        UdpTransport {
            socket_address: socket_address
        }
    }

    fn send(&self, message: Vec<u8>) {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.send_to(&message[..], self.socket_address).unwrap();
    }

    fn serialize(&self) -> messages::protobufs::Transport {
        let mut transport = messages::protobufs::UdpTransport::new();
        match self.socket_address.ip() {
            IpAddr::V4(ipv4_addr) => {
                transport.set_ip_address(ipv4_addr.octets().iter().cloned().collect());
            }
            IpAddr::V6(_) => {
                // TODO ipv6 node support
            }
        }
        transport.set_port(self.socket_address.port() as u32);
        let mut message = messages::protobufs::Transport::new();
        message.set_transport_type(messages::protobufs::Transport_Type::UDP);
        message.set_udp_transport(transport);
        message
    }
}

impl fmt::Debug for UdpTransport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UdpTransport {{ {} }}", self.socket_address)
    }
}

/// A `Node` is a peer in the network. It represents another network participant such as ourself.
/// It has an address, and a means to be sent messages. When we receive messages, they come from
/// other nodes.
///
/// Currently, a node is connected via UDP socket, but in a future iteration, a Node will be
/// decoupled from the underlying connection method. A Node semantically represents a
/// "participant," i.e. a person, robot, agent, etc. Not a single device, or single IP address.
///
/// That participant may be connected to the network via a variety of means. Most common would be
/// over the internet via UDP, but it could very well include more connections via LAN, Bluetooth,
/// shortwave radio, signal lamp, or any other medium that can transmit packets.
///
/// This in an important feature to the goals of comm. The comm network should be able to overlay
/// partitioned networks. If a subnetwork is connected via Bluetooth, and at least one participant
/// has a connection to the larger network, all participants are thereby connected to the larger
/// network.
pub struct Node {
    address: Address,
    transports: HashSet<Transport>,
    pending_queries: HashMap<TransactionId, time::Tm>,
    has_ever_responded: bool,
    last_received_query: time::Tm,
    last_received_response: time::Tm,
}

impl Node {
    /// Deserialize a `Node` from a protobuf.
    pub fn deserialize(message: &messages::protobufs::Node) -> Node {
        let address = Address::from_str(message.get_id()).unwrap();
        let transports = message.get_transports().
            iter().
            map(|t| Transport::deserialize(t)).
            collect();
        Node::new(address, transports)
    }

    pub fn new(address: Address, transports: HashSet<Transport>) -> Node {
        Node {
            address: address,
            transports: transports,
            pending_queries: HashMap::new(),
            has_ever_responded: false,
            last_received_query: time::now_utc(),
            last_received_response: time::now_utc()
        }
    }

    fn status(&self) -> Status {
        let time_since_last_seen = time::now_utc() - self.last_seen();

        if self.has_ever_responded &&
            time_since_last_seen < time::Duration::minutes(MINUTES_UNTIL_QUESTIONABLE) {
            Status::Good
        } else if self.pending_query_count() < FAILED_TO_RESPOND_THRESHOLD {
            Status::Questionable
        } else {
            Status::Bad
        }
    }

    /// Whether the node should be considered bad or unreliable. A bad node has not been heard from
    /// in the last `MINUTES_UNTIL_QUESTIONABLE` minutes, and hasn't responded to at least
    /// `FAILED_TO_RESPOND_THRESHOLD` queries.
    ///
    /// Generally, a bad node SHOULD NOT be sent queries, since it's likely to be a waste of
    /// network traffic. However, the node should not forgotten until its bucket is full, cannot be
    /// split, and a new node is being inserted into it. At that point, it's advantageous to remove
    /// the worst node from the bucket and insert another in its place.
    ///
    /// If a bad node begins responding to queries, it can become good again. The goal is to
    /// minimize wasted network traffic, but also to minimize network volatility. Repeatedly
    /// ejecting and reintroducing flaky nodes makes for a volatile network.
    ///
    /// TODO: Should we forgive a bad node's `pending_query_count()` if it becomes good again?
    pub fn is_bad(&self) -> bool {
        self.status() == Status::Bad
    }

    /// Whether the node is questionable and should therefore be pinged. A questionable node SHOULD
    /// still be sent relevant queries, and SHOULD be given every benefit of being treated as if it
    /// were good. Questionable only means that we don't know whether it's still good, and we
    /// should ask it.
    pub fn is_questionable(&self) -> bool {
        self.status() == Status::Questionable
    }

    /// The last time we received either a query or a response from a node.
    pub fn last_seen(&self) -> time::Tm {
        cmp::max(self.last_received_query, self.last_received_response)
    }

    /// How many unanswered queries we've sent to a node.
    pub fn pending_query_count(&self) -> usize {
        self.pending_queries.len()
    }

    /// Update the `last_received_query` timestamp for a node. Any time the network receives a
    /// query from a node, this method should be called, passing in the query's `TransactionId`.
    ///
    /// The TID is currently unused, but may be useful in the future.
    pub fn received_query(&mut self, transaction_id: TransactionId) {
        self.last_received_query = time::now_utc();
        debug!("Received query {} from {:?}", &transaction_id, &self);
    }

    /// Update the `last_received_response` timestamp if we're indeed waiting for a response to
    /// a query we previously sent with a TID of `transaction_id`.
    ///
    /// If we're not expecting a response from this node for a query we sent previously, this
    /// method call is ignored and the timestamp is left as is.
    pub fn received_response(&mut self, transaction_id: TransactionId) {
        self.last_received_response = time::now_utc();
        if let Some(queried_at) = self.pending_queries.remove(&transaction_id) {
            self.has_ever_responded = true;
            debug!("Received response from {:?} for transaction {} in {}ms",
                   &self, &transaction_id, time::now_utc() - queried_at);
        } else {
            debug!("Received unexpected response from {:?} for transaction {}",
                   &self, transaction_id);
            debug!("  pending queries: {:?}", self.pending_queries.len());
        }
    }

    /// Send an encoded message to a node via its transports. A node may have numerous transports
    /// (aka connections).
    ///
    /// TODO: There should be some logic to determine which transports are still good.
    pub fn send(&self, message: Vec<u8>) {
        for transport in &self.transports {
            transport.send(message.clone());
        }
    }

    /// Records that we're expecting a response from this node for the TID `transaction_id`.
    ///
    /// TODO: Perhaps this method should be rolled together with a method like `send_query` that
    /// records the TID and calls `send`.
    pub fn sent_query(&mut self, transaction_id: TransactionId) {
        self.pending_queries.insert(transaction_id, time::now_utc());
    }

    /// Update the socket address and port of a node. This is useful for when a node disconnects
    /// and reconnects to the internet, or changes IP addresses, etc.
    ///
    /// In the future, a node will have n connections instead of one socket address, and we'd just
    /// add another to the set, and keep track of which are good, eventually ejecting dead
    /// connections.
    ///
    /// TODO: this is a temporary measure until a node can support multiple connections. Rather
    /// than updating its only connection, it should just add another to the set and keep track of
    /// the most reliable connection.
    pub fn update_connection(&mut self, other_node: Self) {
        for transport in other_node.transports {
            self.transports.insert(transport);
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Serialize for Node {
    fn serialize(&self) -> messages::protobufs::Node {
        let mut message = messages::protobufs::Node::new();
        message.set_id(self.address.to_str());
        let transports = self.transports.iter().map(|t| t.serialize()).collect();
        let transports = protobuf::RepeatedField::from_vec(transports);
        message.set_transports(transports);
        message
    }
}

impl Addressable for Node {
    fn address(&self) -> Address {
        self.address
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node {{ {}, {:?} }}", self.address, self.transports)
    }
}

#[cfg(test)]
pub mod tests {
    use address::Address;
    use std::collections::{HashMap, HashSet};
    use std::net::ToSocketAddrs;
    use super::{FAILED_TO_RESPOND_THRESHOLD, MINUTES_UNTIL_QUESTIONABLE, Node, Serialize, Transport, UdpTransport};
    use time;
    use transaction::TransactionId;

    pub fn new(address: Address, last_received_response: time::Tm, pending_queries: HashMap<TransactionId, time::Tm>) -> Node {
        use rand::{thread_rng, Rng};
        let port = thread_rng().gen_range(1000, 10000);
        let mut transports = HashSet::new();
        transports.insert(Transport::Udp(UdpTransport::new(("0.0.0.0", port).to_socket_addrs().unwrap().next().unwrap())));
        Node {
            address: address,
            transports: transports,
            pending_queries: pending_queries,
            has_ever_responded: false,
            last_received_query: time::empty_tm(),
            last_received_response: last_received_response
        }
    }

    pub fn good(address: Address) -> Node {
        new(address, time::now_utc(), HashMap::new())
    }

    pub fn questionable(address: Address) -> Node {
        let last_received_response = time::now_utc() -
            time::Duration::minutes(MINUTES_UNTIL_QUESTIONABLE);

        new(address, last_received_response, HashMap::new())
    }

    pub fn bad(address: Address) -> Node {
        let last_received_response = time::now_utc() -
            time::Duration::minutes(MINUTES_UNTIL_QUESTIONABLE + 1);
        let mut pending_queries = HashMap::new();
        let n_queries = FAILED_TO_RESPOND_THRESHOLD + 1;
        for i in 0..n_queries {
            pending_queries.insert(i as TransactionId, time::now_utc());
        }

        new(address, last_received_response, pending_queries)
    }

    #[test]
    fn test_received_response() {
        let address = Address::for_content("some string");
        let transports = HashSet::new();
        let mut node = Node::new(address, transports);

        // When it's not expecting the response
        node.received_response(1);
        assert!(!node.has_ever_responded);

        // When it's expecting the response
        let last_seen_before = node.last_seen();
        let transaction_id = 2;
        node.sent_query(transaction_id);
        node.received_response(transaction_id);
        let last_seen_after = node.last_seen();
        assert!(last_seen_before < last_seen_after);
        assert!(node.has_ever_responded);
    }

    #[test]
    fn test_serialize() {
        use messages;
        let mut node_message = messages::protobufs::Node::new();
        let mut transport_message = messages::protobufs::Transport::new();
        let mut udp_transport_message = messages::protobufs::UdpTransport::new();
        udp_transport_message.set_ip_address(vec![192, 168, 1, 2]);
        udp_transport_message.set_port(9000);
        transport_message.set_transport_type(messages::protobufs::Transport_Type::UDP);
        transport_message.set_udp_transport(udp_transport_message);
        node_message.set_id("8b45e4bd1c6acb88bebf6407d16205f567e62a3e".to_string());
        node_message.set_transports(protobuf::RepeatedField::from_vec(vec![transport_message]));

        let address = Address::for_content("some string");
        let socket_address = ("192.168.1.2", 9000).to_socket_addrs().unwrap().next().unwrap();
        let mut transports = HashSet::new();
        transports.insert(Transport::Udp(UdpTransport::new(socket_address)));
        let node = Node::new(address, transports);
        assert_eq!(node.serialize(), node_message);
    }
}
