use address::{LENGTH, Address, Addressable};
use num::bigint::{BigUint, ToBigUint};
use std::collections::HashMap;
use num;

struct NodeBucket<A: Addressable> {
    k: usize,
    min_exp: usize,
    max_exp: usize,
    addresses: Vec<Address>,
    nodes: HashMap<Address, A>
}

impl<A: Addressable> NodeBucket<A> {
    pub fn new(k: usize) -> NodeBucket<A> {
        let min_exp = 0;
        let max_exp = LENGTH;
        NodeBucket {
            k: k,
            min_exp: min_exp,
            max_exp: max_exp,
            addresses: Vec::with_capacity(k),
            nodes: HashMap::with_capacity(k)
        }
    }

    pub fn contains(&self, address: &Address) -> bool {
        self.addresses.contains(address)
    }

    pub fn covers(&self, address: &Address) -> bool {
        let numeric = address.as_numeric();
        numeric >= self.min() && numeric <= self.max()
    }

    fn max(&self) -> num::BigUint {
        num::pow(2u8.to_biguint().unwrap(), self.max_exp) - 1.to_biguint().unwrap()
    }

    fn min(&self) -> num::BigUint {
        num::pow(2.to_biguint().unwrap(), self.min_exp) - 1.to_biguint().unwrap()
    }

    pub fn insert(&mut self, node: A) {
        let address = node.address();
        if let Some(pos) = self.addresses.iter().position(|&a| a == address) {
            self.addresses.remove(pos);
            self.addresses.insert(0, address);
            self.nodes.insert(address, node);
        } else if !self.is_full() {
            self.addresses.insert(0, address);
            self.nodes.insert(address, node);
        }
    }

    pub fn is_full(&self) -> bool {
        self.addresses.len() >= self.k
    }

    pub fn split(&mut self) -> (Self, Self) {
        let partition = num::pow(2.to_biguint().unwrap(), self.max_exp - 1);
        let addresses = self.addresses.clone();
        let (a_addresses, b_addresses): (Vec<Address>, Vec<Address>) = addresses
            .into_iter()
            .partition(|&a| a.as_numeric() < partition);
        let mut a_nodes = HashMap::new();
        let mut b_nodes = HashMap::new();
        for (address, node) in self.nodes.drain() {
            if address.as_numeric() < partition {
                a_nodes.insert(address, node);
            } else {
                b_nodes.insert(address, node);
            }
        }

        let a = NodeBucket {
            k: self.k,
            min_exp: 0,
            max_exp: self.max_exp - 1,
            addresses: a_addresses,
            nodes: a_nodes
        };
        let b = NodeBucket {
            k: self.k,
            min_exp: self.max_exp - 1,
            max_exp: LENGTH,
            addresses: b_addresses,
            nodes: b_nodes
        };
        (a, b)
    }
}

#[cfg(test)]
mod tests {
    use address::{Address, Addressable};
    use super::NodeBucket;

    #[derive(Clone,Copy,Debug,PartialEq,Eq)]
    struct Node {
        pub address: Address
    }

    impl Node {
        fn new(address: Address) -> Node {
            Node { address: address }
        }
    }

    impl Addressable for Node {
        fn address(&self) -> Address {
            self.address
        }
    }

    #[test]
    fn test_insert() {
        let mut bucket: NodeBucket<Node> = NodeBucket::new(8);
        let node = Node::new(Address::for_content("some string"));
        bucket.insert(node);
        assert_eq!(bucket.addresses.len(), 1);
    }

    #[test]
    fn test_insert_duplicate() {
        let mut bucket: NodeBucket<Node> = NodeBucket::new(8);
        let a = Node::new(Address::for_content("node 1"));
        let b = Node::new(Address::for_content("node 2"));
        let c = Node::new(Address::for_content("node 1"));

        bucket.insert(a.clone());
        bucket.insert(b);
        bucket.insert(c);

        assert_eq!(bucket.addresses.len(), 2);
        assert_eq!(bucket.nodes[&Address::for_content("node 1")], a);
    }

    #[test]
    fn test_insert_full() {
        let mut bucket: NodeBucket<Node> = NodeBucket::new(2);
        bucket.insert(Node::new(Address::for_content("node 1")));
        bucket.insert(Node::new(Address::for_content("node 2")));
        bucket.insert(Node::new(Address::for_content("node 3")));
        assert_eq!(bucket.addresses.len(), 2);
        assert_eq!(bucket.nodes.get(&Address::for_content("node 3")), None);
    }

    #[test]
    fn test_is_full() {
        let mut bucket: NodeBucket<Node> = NodeBucket::new(2);
        assert!(!bucket.is_full());

        bucket.insert(Node::new(Address::for_content("node 1")));
        bucket.insert(Node::new(Address::for_content("node 2")));
        assert!(bucket.is_full());
    }

    #[test]
    fn test_contains() {
        let mut bucket: NodeBucket<Node> = NodeBucket::new(8);
        bucket.insert(Node::new(Address::for_content("node 1")));
        assert!(bucket.contains(&Address::for_content("node 1")));
    }

    #[test]
    fn test_covers() {
        let mut bucket: NodeBucket<Node> = NodeBucket::new(8);
        let address_1 = Address::from_str("0000000000000000000000000000000000000000");
        let address_2 = Address::from_str("ffffffffffffffffffffffffffffffffffffffff");
        assert!(bucket.covers(&address_1));
        assert!(bucket.covers(&address_2));
    }

    #[test]
    fn test_split() {
        let mut bucket: NodeBucket<Node> = NodeBucket::new(2);
        let node_1 = Node::new(Address::from_str("0000000000000000000000000000000000000000"));
        let node_2 = Node::new(Address::from_str("ffffffffffffffffffffffffffffffffffffffff"));
        bucket.insert(node_1);
        bucket.insert(node_2);
        let (a, b) = bucket.split();
        assert_eq!(a.addresses.len(), 1);
        assert_eq!(b.addresses.len(), 1);
        assert_eq!(a.nodes[&Address::from_str("0000000000000000000000000000000000000000")], node_1);
        assert_eq!(b.nodes[&Address::from_str("ffffffffffffffffffffffffffffffffffffffff")], node_2);
        assert!(a.covers(&Address::from_str("0000000000000000000000000000000000000000")));
        assert!(!a.covers(&Address::from_str("ffffffffffffffffffffffffffffffffffffffff")));
        assert!(!b.covers(&Address::from_str("0000000000000000000000000000000000000000")));
        assert!(b.covers(&Address::from_str("ffffffffffffffffffffffffffffffffffffffff")));
    }
}
