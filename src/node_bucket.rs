use address::{LENGTH, Addressable, Address};
use node::Node;
use num::bigint::{BigUint, ToBigUint};
use std::cmp;
use std::collections::HashMap;
use std::fmt;
use time;
use num;

const MINUTES_UNTIL_NEEDS_REFRESH: i64 = 15;

#[derive(Debug, PartialEq)]
pub enum InsertOutcome {
    Inserted,   // Inserted new node
    Updated,    // Updated existing node
    Discarded   // Bucket is full
}

pub type InsertionResult = Result<InsertOutcome, String>;

pub struct NodeBucket {
    k: usize,
    min: BigUint,
    max: BigUint,
    addresses: Vec<Address>,
    nodes: HashMap<Address, Node>,
    last_inserted: time::Tm
}

impl fmt::Debug for NodeBucket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NodeBucket {{ [{:040x} - {:040x}], contains: {:?} last_changed: {:?}}}",
               self.min, self.max, self.addresses, self.last_changed())
    }
}

impl NodeBucket {
    pub fn new(k: usize) -> NodeBucket {
        let min = 0.to_biguint().unwrap();
        let max = num::pow(2.to_biguint().unwrap(), LENGTH);
        NodeBucket {
            k: k,
            min: min,
            max: max,
            addresses: Vec::with_capacity(k),
            nodes: HashMap::with_capacity(k),
            last_inserted: time::empty_tm()
        }
    }

    pub fn any_bad_nodes(&self) -> bool {
        self.nodes.iter().any(|(_, node)| node.is_bad())
    }

    pub fn contains(&self, address: &Address) -> bool {
        self.addresses.contains(address)
    }

    pub fn covers(&self, address: &Address) -> bool {
        let numeric = address.as_numeric();
        self.min <= numeric && numeric < self.max
    }

    pub fn find_node(&mut self, address: &Address) -> Option<&mut Node> {
        self.nodes.get_mut(address)
    }

    pub fn get_nodes(&mut self) -> Vec<&mut Node> {
        self.nodes.iter_mut().map(|(_, node)| node).collect()
    }

    pub fn questionable_nodes(&mut self) -> Vec<&mut Node> {
        self.nodes.iter_mut()
            .map(|(_, node)| node)
            .filter(|n| n.is_questionable())
            .collect()
    }

    pub fn insert(&mut self, node: Node) -> InsertionResult {
        let address = node.address();
        if self.covers(&address) {
            if let Some(pos) = self.addresses.iter().position(|a| a == &address) {
                // TODO: Should this update the node's socket address incase a device has changed
                // IPs?
                self.addresses.remove(pos);
                self.addresses.insert(0, address);
                self.last_inserted = time::now_utc();
                Ok(InsertOutcome::Updated)
            } else if !self.is_full() {
                self.addresses.insert(0, address);
                self.nodes.insert(address, node);
                self.last_inserted = time::now_utc();
                Ok(InsertOutcome::Inserted)
            } else if self.any_bad_nodes() {
                self.remove_worst_node();
                self.insert(node)
            } else {
                Ok(InsertOutcome::Discarded)
            }
        } else {
            Err(format!("Bucket {:?}  does not cover {:?}", self, address))
        }
    }

    pub fn is_full(&self) -> bool {
        self.nodes.len() >= self.k
    }

    pub fn last_changed(&self) -> time::Tm {
        if let Some((_, node)) = self.nodes.iter().max_by_key(|&(_, node)| node.last_seen()) {
            cmp::max(self.last_inserted, node.last_seen())
        } else {
            self.last_inserted
        }
    }

    pub fn needs_refresh(&self) -> bool {
        let time_since_changed = time::now_utc() - self.last_changed();
        time_since_changed > time::Duration::minutes(MINUTES_UNTIL_NEEDS_REFRESH)
    }

    pub fn random_address_in_space(&self) -> Address {
        Address::random(&self.min, &self.max)
    }

    fn remove(&mut self, address: &Address) {
        self.nodes.remove(address);
        let pos = self.addresses.iter().position(|a| a == address).unwrap();
        self.addresses.remove(pos);
    }

    pub fn remove_worst_node(&mut self) {
        if let Some(to_remove) = self.nodes.iter()
                .filter(|&(_, n)| n.is_bad())
                .max_by_key(|&(_, n)| n.pending_query_count())
                .map(|(a, _)| a.clone()) {
            self.remove(&to_remove)
        }
    }

    pub fn split(self) -> (Self, Self) {
        let difference = &self.max - &self.min;
        let partition = difference / 2.to_biguint().unwrap() + &self.min;
        let (a_addresses, b_addresses) = self.addresses
            .into_iter()
            .partition(|&a| a.as_numeric() < partition);
        let mut a_nodes = HashMap::new();
        let mut b_nodes = HashMap::new();
        for (address, node) in self.nodes.into_iter() {
            if address.as_numeric() < partition {
                a_nodes.insert(address, node);
            } else {
                b_nodes.insert(address, node);
            }
        }

        let a = NodeBucket {
            k: self.k,
            min: self.min,
            max: partition.clone(),
            addresses: a_addresses,
            nodes: a_nodes,
            last_inserted: self.last_inserted
        };
        let b = NodeBucket {
            k: self.k,
            min: partition,
            max: self.max,
            addresses: b_addresses,
            nodes: b_nodes,
            last_inserted: self.last_inserted
        };
        (a, b)
    }
}

#[cfg(test)]
mod tests {
    use address::{Addressable, Address};
    use node;
    use super::{InsertOutcome, NodeBucket};
    use time;

    #[test]
    fn test_insert() {
        let mut bucket: NodeBucket = NodeBucket::new(8);
        let node = node::tests::good(Address::for_content("some string"));
        assert_eq!(bucket.insert(node).unwrap(), InsertOutcome::Inserted);
        assert_eq!(bucket.addresses.len(), 1);
    }

    #[test]
    fn test_insert_duplicate() {
        let mut bucket: NodeBucket = NodeBucket::new(8);
        let addr_1 = Address::for_content("node 1");
        let addr_2 = Address::for_content("node 2");
        let addr_3 = Address::for_content("node 1");
        let a = node::tests::good(addr_1);
        let b = node::tests::good(addr_2);
        let c = node::tests::good(addr_3);

        bucket.insert(a).unwrap();
        bucket.insert(b).unwrap();
        assert_eq!(bucket.insert(c).unwrap(), InsertOutcome::Updated);

        assert_eq!(bucket.addresses.len(), 2);
        assert_eq!(bucket.nodes[&Address::for_content("node 1")].address(), addr_1);
    }

    #[test]
    fn test_insert_full() {
        let mut bucket: NodeBucket = NodeBucket::new(2);
        bucket.insert(node::tests::good(Address::for_content("node 1"))).unwrap();
        bucket.insert(node::tests::good(Address::for_content("node 2"))).unwrap();
        let result = bucket.insert(node::tests::good(Address::for_content("node 3"))).unwrap();
        assert_eq!(result, InsertOutcome::Discarded);
        assert_eq!(bucket.addresses.len(), 2);
        assert_eq!(bucket.nodes.get(&Address::for_content("node 3")), None);
    }

    #[test]
    fn test_insert_full_with_bad_nodes() {
        let mut bucket: NodeBucket = NodeBucket::new(2);
        bucket.insert(node::tests::good(Address::for_content("node 1"))).unwrap();
        bucket.insert(node::tests::bad(Address::for_content("node 2"))).unwrap();
        // It should eject the bad node and insert this new one
        let result = bucket.insert(node::tests::good(Address::for_content("node 3"))).unwrap();
        assert_eq!(result, InsertOutcome::Inserted);
        assert_eq!(bucket.addresses.len(), 2);
        assert_eq!(bucket.nodes.get(&Address::for_content("node 2")), None);
        assert!(bucket.nodes.get(&Address::for_content("node 3")) != None);
    }

    #[test]
    fn insert_node_outside_of_address_space() {
        use num::bigint::ToBigUint;
        use std::collections::HashMap;

        let mut bucket = NodeBucket {
            k: 8,
            min: 0.to_biguint().unwrap(),
            max: 1.to_biguint().unwrap(),
            addresses: vec![],
            nodes: HashMap::new(),
            last_inserted: time::empty_tm()
        };
        let result = bucket.insert(node::tests::good(Address::for_content("node 3")));
        assert!(result.is_err());
    }

    #[test]
    fn test_is_full() {
        let mut bucket: NodeBucket = NodeBucket::new(2);
        assert!(!bucket.is_full());

        bucket.insert(node::tests::good(Address::for_content("node 1"))).unwrap();
        bucket.insert(node::tests::good(Address::for_content("node 2"))).unwrap();
        assert!(bucket.is_full());
    }

    #[test]
    fn test_contains() {
        let mut bucket: NodeBucket = NodeBucket::new(8);
        bucket.insert(node::tests::good(Address::for_content("node 1"))).unwrap();
        assert!(bucket.contains(&Address::for_content("node 1")));
    }

    #[test]
    fn test_covers() {
        let bucket: NodeBucket = NodeBucket::new(8);
        let address_1 = Address::from_str("0000000000000000000000000000000000000000").unwrap();
        let address_2 = Address::from_str("ffffffffffffffffffffffffffffffffffffffff").unwrap();
        assert!(bucket.covers(&address_1));
        assert!(bucket.covers(&address_2));
    }

    #[test]
    fn test_split() {
        let mut bucket: NodeBucket = NodeBucket::new(4);
        let addr_1 = Address::from_str("0000000000000000000000000000000000000000").unwrap();
        let addr_2 = Address::from_str("7fffffffffffffffffffffffffffffffffffffff").unwrap();
        let addr_3 = Address::from_str("8000000000000000000000000000000000000000").unwrap();
        let addr_4 = Address::from_str("ffffffffffffffffffffffffffffffffffffffff").unwrap();
        let node_1 = node::tests::good(addr_1);
        let node_2 = node::tests::good(addr_2);
        let node_3 = node::tests::good(addr_3);
        let node_4 = node::tests::good(addr_4);

        bucket.insert(node_1).unwrap();
        bucket.insert(node_2).unwrap();
        bucket.insert(node_3).unwrap();
        bucket.insert(node_4).unwrap();

        let (a, b) = bucket.split();

        // Splits up known address
        assert_eq!(a.addresses.len(), 2);
        assert_eq!(b.addresses.len(), 2);

        // Splits up known nodes
        assert_eq!(a.nodes[&Address::from_str("0000000000000000000000000000000000000000").unwrap()].address(), addr_1);
        assert_eq!(a.nodes[&Address::from_str("0000000000000000000000000000000000000000").unwrap()].address(), addr_1);
        assert_eq!(a.nodes[&Address::from_str("7fffffffffffffffffffffffffffffffffffffff").unwrap()].address(), addr_2);
        assert_eq!(b.nodes[&Address::from_str("8000000000000000000000000000000000000000").unwrap()].address(), addr_3);
        assert_eq!(b.nodes[&Address::from_str("ffffffffffffffffffffffffffffffffffffffff").unwrap()].address(), addr_4);

        // Equitably covers address space
        assert!(a.covers(&Address::from_str("0000000000000000000000000000000000000000").unwrap()));
        assert!(!b.covers(&Address::from_str("0000000000000000000000000000000000000000").unwrap()));
        assert!(a.covers(&Address::from_str("7fffffffffffffffffffffffffffffffffffffff").unwrap()));
        assert!(!b.covers(&Address::from_str("7fffffffffffffffffffffffffffffffffffffff").unwrap()));
        assert!(!a.covers(&Address::from_str("8000000000000000000000000000000000000000").unwrap()));
        assert!(b.covers(&Address::from_str("8000000000000000000000000000000000000000").unwrap()));
        assert!(!a.covers(&Address::from_str("ffffffffffffffffffffffffffffffffffffffff").unwrap()));
        assert!(b.covers(&Address::from_str("ffffffffffffffffffffffffffffffffffffffff").unwrap()));
    }

    #[test]
    fn test_last_changed() {
        let mut bucket: NodeBucket = NodeBucket::new(4);

        // it updates upon inserting a node
        let last_changed_before_insert = bucket.last_changed();
        let addr = Address::from_str("0000000000000000000000000000000000000000").unwrap();
        let node = node::tests::good(addr);
        bucket.insert(node).unwrap();
        let last_changed_after_insert = bucket.last_changed();
        assert!(last_changed_before_insert < last_changed_after_insert);

        // it updates when one of its nodes are heard from
        let last_changed_before_received = bucket.last_changed();
        bucket.find_node(&addr).unwrap().received_response(1);
        let last_changed_after_received = bucket.last_changed();
        assert!(last_changed_before_received < last_changed_after_received);
    }

    #[test]
    fn test_remove_worst_node() {
        let mut bucket: NodeBucket = NodeBucket::new(8);
        let node_1 = node::tests::good(Address::for_content("good node"));
        let node_2 = node::tests::questionable(Address::for_content("questionable node"));
        let node_3 = node::tests::bad(Address::for_content("bad node"));
        bucket.insert(node_1).unwrap();
        bucket.insert(node_2).unwrap();
        bucket.insert(node_3).unwrap();
        assert_eq!(bucket.addresses.len(), 3);

        bucket.remove_worst_node();
        assert_eq!(bucket.addresses.len(), 2);
    }
}
