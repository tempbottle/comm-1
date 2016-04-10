use address::{LENGTH, Address, Addressable};
use num::bigint::{BigUint, ToBigUint};
use std::collections::HashMap;
use num;

pub struct NodeBucket<A: Addressable> {
    k: usize,
    min: BigUint,
    max: BigUint,
    addresses: Vec<Address>,
    nodes: HashMap<Address, A>
}

impl<A: Addressable> NodeBucket<A> {
    pub fn new(k: usize) -> NodeBucket<A> {
        let min = 0.to_biguint().unwrap();
        let max = num::pow(2.to_biguint().unwrap(), LENGTH);
        NodeBucket {
            k: k,
            min: min,
            max: max,
            addresses: Vec::with_capacity(k),
            nodes: HashMap::with_capacity(k)
        }
    }

    pub fn contains(&self, address: &Address) -> bool {
        self.addresses.contains(address)
    }

    pub fn covers(&self, address: &Address) -> bool {
        let numeric = address.as_numeric();
        self.min <= numeric && numeric < self.max
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
        let partition = self.max.clone() / 2.to_biguint().unwrap();
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
            min: self.min.clone(),
            max: partition.clone(),
            addresses: a_addresses,
            nodes: a_nodes
        };
        let b = NodeBucket {
            k: self.k,
            min: partition.clone(),
            max: self.max.clone(),
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
        let bucket: NodeBucket<Node> = NodeBucket::new(8);
        let address_1 = Address::from_str("0000000000000000000000000000000000000000");
        let address_2 = Address::from_str("ffffffffffffffffffffffffffffffffffffffff");
        assert!(bucket.covers(&address_1));
        assert!(bucket.covers(&address_2));
    }

    #[test]
    fn test_split() {
        let mut bucket: NodeBucket<Node> = NodeBucket::new(4);
        let node_1 = Node::new(Address::from_str("0000000000000000000000000000000000000000"));
        let node_2 = Node::new(Address::from_str("7fffffffffffffffffffffffffffffffffffffff"));
        let node_3 = Node::new(Address::from_str("8000000000000000000000000000000000000000"));
        let node_4 = Node::new(Address::from_str("ffffffffffffffffffffffffffffffffffffffff"));

        bucket.insert(node_1);
        bucket.insert(node_2);
        bucket.insert(node_3);
        bucket.insert(node_4);

        let (a, b) = bucket.split();

        // Splits up known address
        assert_eq!(a.addresses.len(), 2);
        assert_eq!(b.addresses.len(), 2);

        // Splits up known nodes
        assert_eq!(a.nodes[&Address::from_str("0000000000000000000000000000000000000000")], node_1);
        assert_eq!(a.nodes[&Address::from_str("7fffffffffffffffffffffffffffffffffffffff")], node_2);
        assert_eq!(b.nodes[&Address::from_str("8000000000000000000000000000000000000000")], node_3);
        assert_eq!(b.nodes[&Address::from_str("ffffffffffffffffffffffffffffffffffffffff")], node_4);

        // Equitably covers address space
        assert!(a.covers(&Address::from_str("0000000000000000000000000000000000000000")));
        assert!(!b.covers(&Address::from_str("0000000000000000000000000000000000000000")));
        assert!(a.covers(&Address::from_str("7fffffffffffffffffffffffffffffffffffffff")));
        assert!(!b.covers(&Address::from_str("7fffffffffffffffffffffffffffffffffffffff")));
        assert!(!a.covers(&Address::from_str("8000000000000000000000000000000000000000")));
        assert!(b.covers(&Address::from_str("8000000000000000000000000000000000000000")));
        assert!(!a.covers(&Address::from_str("ffffffffffffffffffffffffffffffffffffffff")));
        assert!(b.covers(&Address::from_str("ffffffffffffffffffffffffffffffffffffffff")));
    }
}
