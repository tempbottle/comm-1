use std::net::ToSocketAddrs;
use time;

struct UdpNode<A: ToSocketAddrs> {
    address: A,
    last_seen: time::Tm
}

trait Node<A: ToSocketAddrs> {
    fn new(address: A) -> Self;
    fn update(&mut self);
}

impl<A: ToSocketAddrs> Node<A> for UdpNode<A> {
    fn new(address: A) -> UdpNode<A> {
        UdpNode {
            address: address,
            last_seen: time::now_utc()
        }
    }

    fn update(&mut self) {
        self.last_seen = time::now_utc()
    }
}

#[cfg(test)]
mod tests {
    use super::{Node,UdpNode};

    #[test]
    fn test_update() {
        let mut node = UdpNode::new(("0.0.0.0", 9000));
        let last_seen_before = node.last_seen;
        node.update();
        let last_seen_after = node.last_seen;
        assert!(last_seen_before < last_seen_after);
    }
}
