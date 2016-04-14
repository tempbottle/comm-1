use node_bucket::NodeBucket;
use address::{Address, Addressable, LENGTH};

#[derive(Debug)]
pub struct RoutingTable<A: Addressable> {
    k: usize,
    self_address: Address,
    buckets: Vec<NodeBucket<A>>
}

impl<A: Addressable> RoutingTable<A> {
    pub fn new(k: usize, self_address: Address) -> RoutingTable<A> {
        let bucket = NodeBucket::new(k);
        RoutingTable {
            k: k,
            self_address: self_address,
            buckets: vec![bucket]
        }
    }

    pub fn insert(&mut self, node: A) {
        let index = self.bucket_for(&node.get_address());
        let mut bucket = self.buckets.remove(index);
        let self_address = self.self_address;

        if !self.buckets_maxed() && bucket.is_full() && bucket.covers(&self_address) {
            let (a, b) = bucket.split();
            self.buckets.insert(index, a);
            self.buckets.insert(index + 1, b);
            self.insert(node);
        } else {
            bucket.insert(node);
            self.buckets.insert(index, bucket);
        }
    }

    pub fn nearest_to(&mut self, address: &Address) -> Vec<&mut A> {
        // TODO: this should walk buckets much more efficiently

        let mut candidates: Vec<&mut A> = self.buckets
            .iter_mut()
            .flat_map(|b| b.get_nodes())
            .collect();

        candidates.sort_by_key(|n| n.get_address().distance_from(address));

        candidates.into_iter().take(self.k).collect()
    }

    fn bucket_for(&mut self, address: &Address) -> usize {
        let (index, _) = self.buckets
            .iter()
            .enumerate()
            .find(|&(_, ref b)| b.covers(address))
            .unwrap();
        index
    }

    fn buckets_maxed(&self) -> bool {
        self.buckets.len() >= LENGTH
    }
}

#[cfg(test)]
mod tests {
    use super::RoutingTable;
    use address::{Address, Addressable};

    #[derive(Clone,Copy,Debug,PartialEq,Eq)]
    struct TestNode {
        pub address: Address
    }

    impl TestNode {
        fn new(address: Address) -> TestNode {
            TestNode { address: address }
        }
    }

    impl Addressable for TestNode {
        fn get_address(&self) -> Address {
            self.address
        }
    }

    #[test]
    fn test_insert() {
        let self_node = Address::from_str("0000000000000000000000000000000000000000");
        let mut table: RoutingTable<TestNode> = RoutingTable::new(2, self_node);
        table.insert(TestNode::new(Address::from_str("0000000000000000000000000000000000000001")));
        table.insert(TestNode::new(Address::from_str("ffffffffffffffffffffffffffffffffffffffff")));
        assert_eq!(table.buckets.len(), 1);

        // Splits buckets upon adding a k+1th node in the same space as self node
        table.insert(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffe")));
        assert_eq!(table.buckets.len(), 2);
        table.insert(TestNode::new(Address::from_str("7fffffffffffffffffffffffffffffffffffffff")));
        table.insert(TestNode::new(Address::from_str("7ffffffffffffffffffffffffffffffffffffffe")));
        assert_eq!(table.buckets.len(), 3);

        // Replaces instead of duplicates existing nodes
        table.insert(TestNode::new(Address::from_str("0000000000000000000000000000000000000001")));
        table.insert(TestNode::new(Address::from_str("0000000000000000000000000000000000000001")));
        table.insert(TestNode::new(Address::from_str("0000000000000000000000000000000000000001")));
        assert_eq!(table.buckets.len(), 3);

        // Disregards new nodes for full, non-self space buckets
        table.insert(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffd")));
        table.insert(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffc")));
        table.insert(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffb")));
        assert_eq!(table.buckets.len(), 3);
    }

    #[test]
    fn test_nearest_to() {
        let self_node = Address::from_str("0000000000000000000000000000000000000000");
        let mut table: RoutingTable<TestNode> = RoutingTable::new(2, self_node);
        let node_1 = TestNode::new(Address::from_str("0000000000000000000000000000000000000001"));
        let node_2 = TestNode::new(Address::from_str("7ffffffffffffffffffffffffffffffffffffffe"));
        let node_3 = TestNode::new(Address::from_str("ffffffffffffffffffffffffffffffffffffffff"));
        table.insert(node_1);
        table.insert(node_2);
        table.insert(node_3);

        {
            let nearest = table.nearest_to(&Address::from_str("fffffffffffffffffffffffffffffffffffffffd"));
            assert_eq!(*nearest[0], node_3);
            assert_eq!(*nearest[1], node_2);
        }

        {
            let nearest = table.nearest_to(&Address::from_str("0000000000000000000000000000000000000002"));
            assert_eq!(*nearest[0], node_1);
            assert_eq!(*nearest[1], node_2);
        }
    }
}
