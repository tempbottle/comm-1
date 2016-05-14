use address::{Address, Addressable};
use messages;
use std::fmt::Debug;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket, IpAddr, Ipv4Addr};
use std::collections::HashMap;
use time;
use transaction::TransactionId;

const FAILED_TO_RESPOND_THRESHOLD: usize = 3;
const MINUTES_UNTIL_QUESTIONABLE: i64 = 1;

pub fn deserialize(message: &messages::protobufs::Node) -> Box<Node> {
    let ip = message.get_ip_address();
    let ip = Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
    let port = message.get_port() as u16;
    let address = Address::from_str(message.get_id());
    Box::new(UdpNode::new(address, (ip, port)))
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

pub trait Node : Addressable + Debug + Serialize + Send + Sync {
    fn is_questionable(&self) -> bool;
    fn received_query(&mut self, transaction_id: TransactionId);
    fn received_response(&mut self, transaction_id: TransactionId);
    fn send(&self, message: Vec<u8>);
    fn sent_query(&mut self, transaction_id: TransactionId);
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.get_address() == other.get_address()
    }
}

#[derive(Debug)]
pub struct UdpNode {
    address: Address,
    socket_address: SocketAddr,
    pending_queries: HashMap<TransactionId, time::Tm>,
    has_ever_responded: bool,
    last_seen: time::Tm,
}

impl UdpNode {
    pub fn new<S: ToSocketAddrs>(address: Address, socket_address: S) -> UdpNode {
        let socket_address = socket_address
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        UdpNode {
            address: address,
            socket_address: socket_address,
            pending_queries: HashMap::new(),
            has_ever_responded: false,
            last_seen: time::empty_tm(),
        }
    }

    fn status(&self) -> Status {
        let time_since_last_seen = time::now_utc() - self.last_seen;

        if self.has_ever_responded &&
            time_since_last_seen < time::Duration::minutes(MINUTES_UNTIL_QUESTIONABLE) {
            Status::Good
        } else if self.pending_queries.len() < FAILED_TO_RESPOND_THRESHOLD {
            Status::Questionable
        } else {
            Status::Bad
        }
    }
}

impl Node for UdpNode {
    fn is_questionable(&self) -> bool {
        self.status() == Status::Questionable
    }

    fn received_query(&mut self, _: TransactionId) {
        self.last_seen = time::now_utc();
    }

    fn received_response(&mut self, transaction_id: TransactionId) {
        self.last_seen = time::now_utc();
        if let Some(queried_at) = self.pending_queries.remove(&transaction_id) {
            self.has_ever_responded = true;
            let time_to_respond = time::now_utc() - queried_at;
            println!("{} took {:?} to respond", transaction_id, time_to_respond);
        } else {
            println!("{:?} was not expecting response to {}", self.address, transaction_id);
            println!("pending queries: {:?}", self.pending_queries.len());
        }
    }

    fn send(&self, message: Vec<u8>) {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.send_to(&message[..], self.socket_address).unwrap();
    }

    fn sent_query(&mut self, transaction_id: TransactionId) {
        self.pending_queries.insert(transaction_id, time::now_utc());
    }
}

impl Serialize for UdpNode {
    fn serialize(&self) -> messages::protobufs::Node {
        let mut message = messages::protobufs::Node::new();
        message.set_id(self.address.to_str());
        match self.socket_address.ip() {
            IpAddr::V4(ipv4_addr) => {
                message.set_ip_address(ipv4_addr.octets().iter().cloned().collect());
            }
            IpAddr::V6(ipv6_addr) => {
                // TODO ipv6 node support
            }
        }
        message.set_port(self.socket_address.port() as u32);
        message
    }
}

impl Addressable for UdpNode {
    fn get_address(&self) -> Address {
        self.address
    }
}

#[cfg(test)]
mod tests {
    use address::Address;
    use super::{Node, UdpNode, Serialize};

    #[test]
    fn test_received_response() {
        let address = Address::for_content("some string");
        let mut node = UdpNode::new(address, ("0.0.0.0", 9000));

        // When it's not expecting the response
        node.received_response(1);
        assert!(!node.has_ever_responded);

        // When it's expecting the response
        let last_seen_before = node.last_seen;
        let transaction_id = 2;
        node.sent_query(transaction_id);
        node.received_response(transaction_id);
        let last_seen_after = node.last_seen;
        assert!(last_seen_before < last_seen_after);
        assert!(node.has_ever_responded);
    }

    #[test]
    fn test_serialize() {
        use messages;
        use protobuf::Message;
        let mut protobuf = messages::protobufs::Node::new();
        protobuf.set_id("8b45e4bd1c6acb88bebf6407d16205f567e62a3e".to_string());
        protobuf.set_ip_address(vec![192, 168, 1, 2]);
        protobuf.set_port(9000);
        let address = Address::for_content("some string");
        let node = UdpNode::new(address, ("192.168.1.2", 9000));
        assert_eq!(node.serialize(), protobuf);
    }
}
