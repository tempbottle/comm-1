use address::{Address, Addressable};
use messages;
use protobuf::Message;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket, Ipv4Addr};
use time;

#[derive(Debug)]
pub struct UdpNode {
    address: Address,
    socket_address: SocketAddr,
    last_seen: time::Tm
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
            last_seen: time::now_utc()
        }
    }
}


pub trait Node {
    fn update(&mut self);
    fn send<M: Message>(&self, message: M);
}

impl Node for UdpNode {
    fn update(&mut self) {
        self.last_seen = time::now_utc()
    }

    fn send<M: Message>(&self, message: M) {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let ref buf = message.write_to_bytes().unwrap()[..];
        socket.send_to(buf, self.socket_address).unwrap();
    }
}

pub trait Serialization {
    fn deserialize(message: &messages::Node) -> Self;
    fn serialize(&self) -> messages::Node;
}

impl Serialization for UdpNode {
    fn deserialize(message: &messages::Node) -> UdpNode {
        let ip = message.get_ip_address();
        let ip = Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
        let port = message.get_port() as u16;
        let address = Address::from_str(message.get_id());
        Self::new(address, (ip, port))
    }

    fn serialize(&self) -> messages::Node {
        let mut message = messages::Node::new();
        message.set_id(self.address.to_str());
        message.set_ip_address(vec![127, 0, 0, 1]); // TODO: use actual IP address
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
    use super::{Node, UdpNode};
    use address::Address;

    #[test]
    fn test_update() {
        let address = Address::for_content("some string");
        let mut node = UdpNode::new(address, ("0.0.0.0", 9000));
        let last_seen_before = node.last_seen;
        node.update();
        let last_seen_after = node.last_seen;
        assert!(last_seen_before < last_seen_after);
    }
}
