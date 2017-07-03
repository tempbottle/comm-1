use address::{Address, Addressable};
use messages;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket, IpAddr, Ipv4Addr};
use std::collections::HashMap;
use std::cmp;
use time;
use transaction::TransactionId;

pub const FAILED_TO_RESPOND_THRESHOLD: usize = 5;
pub const MINUTES_UNTIL_QUESTIONABLE: i64 = 15;

pub fn deserialize(message: &messages::protobufs::Node) -> Node {
    let ip = message.get_ip_address();
    let ip = Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
    let port = message.get_port() as u16;
    let address = Address::from_str(message.get_id()).unwrap();
    Node::new(address, (ip, port))
}

pub trait Serialize {
    fn serialize(&self) -> messages::protobufs::Node;
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Good,
    Questionable,
    Bad
}

#[derive(Debug)]
pub struct Node {
    address: Address,
    socket_address: SocketAddr,
    pending_queries: HashMap<TransactionId, time::Tm>,
    has_ever_responded: bool,
    last_received_query: time::Tm,
    last_received_response: time::Tm,
}

impl Node {
    pub fn new<S: ToSocketAddrs>(address: Address, socket_address: S) -> Node {
        let socket_address = socket_address
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        Node {
            address: address,
            socket_address: socket_address,
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

    pub fn is_bad(&self) -> bool {
        self.status() == Status::Bad
    }

    pub fn is_questionable(&self) -> bool {
        self.status() == Status::Questionable
    }

    pub fn last_seen(&self) -> time::Tm {
        cmp::max(self.last_received_query, self.last_received_response)
    }

    pub fn pending_query_count(&self) -> usize {
        self.pending_queries.len()
    }

    pub fn received_query(&mut self, _: TransactionId) {
        self.last_received_query = time::now_utc();
    }

    pub fn received_response(&mut self, transaction_id: TransactionId) {
        self.last_received_response = time::now_utc();
        if let Some(_queried_at) = self.pending_queries.remove(&transaction_id) {
            self.has_ever_responded = true;
            //let time_to_respond = time::now_utc() - queried_at;
            //debug!("{} took {:?} to respond", transaction_id, time_to_respond);
        } else {
            debug!("{:?} was not expecting response to {}", self.address, transaction_id);
            debug!("pending queries: {:?}", self.pending_queries.len());
        }
    }

    pub fn send(&self, message: Vec<u8>) {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.send_to(&message[..], self.socket_address).unwrap();
    }

    pub fn sent_query(&mut self, transaction_id: TransactionId) {
        self.pending_queries.insert(transaction_id, time::now_utc());
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
        match self.socket_address.ip() {
            IpAddr::V4(ipv4_addr) => {
                message.set_ip_address(ipv4_addr.octets().iter().cloned().collect());
            }
            IpAddr::V6(_) => {
                // TODO ipv6 node support
            }
        }
        message.set_port(self.socket_address.port() as u32);
        message
    }
}

impl Addressable for Node {
    fn address(&self) -> Address {
        self.address
    }
}

#[cfg(test)]
pub mod tests {
    use address::Address;
    use std::collections::HashMap;
    use std::net::ToSocketAddrs;
    use super::{FAILED_TO_RESPOND_THRESHOLD, MINUTES_UNTIL_QUESTIONABLE, Node, Serialize};
    use time;
    use transaction::TransactionId;

    pub fn new(address: Address, last_received_response: time::Tm, pending_queries: HashMap<TransactionId, time::Tm>) -> Node {
        use rand::{thread_rng, Rng};
        let port = thread_rng().gen_range(1000, 10000);
        Node {
            address: address,
            socket_address: ("0.0.0.0", port).to_socket_addrs().unwrap().next().unwrap(),
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
        let mut node = Node::new(address, ("0.0.0.0", 9000));

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
        let mut protobuf = messages::protobufs::Node::new();
        protobuf.set_id("8b45e4bd1c6acb88bebf6407d16205f567e62a3e".to_string());
        protobuf.set_ip_address(vec![192, 168, 1, 2]);
        protobuf.set_port(9000);
        let address = Address::for_content("some string");
        let node = Node::new(address, ("192.168.1.2", 9000));
        assert_eq!(node.serialize(), protobuf);
    }
}
