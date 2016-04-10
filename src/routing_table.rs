use node_bucket::NodeBucket;
use address::{Address, Addressable, LENGTH};

pub struct RoutingTable<A: Addressable> {
    self_address: Address,
    buckets: Vec<NodeBucket<A>>
}

impl<A: Addressable> RoutingTable<A> {
    pub fn new(k: usize, self_address: Address) -> RoutingTable<A> {
        let bucket = NodeBucket::new(k);
        RoutingTable {
            self_address: self_address,
            buckets: vec![bucket]
        }
    }

    pub fn insert(&mut self, node: A) {
        self.bucket_for(&node.address(), {move |bucket|
            bucket.insert(node)
        });
    }

    fn bucket_for<F>(&mut self, address: &Address, func: F)
        where F : FnOnce(&mut NodeBucket<A>) -> () {
        let index: usize;
        let split: bool;
        let buckets_maxed = self.buckets_maxed();
        let self_address = self.self_address;
        {
            let (idx, bucket) = self.buckets
                .iter()
                .enumerate()
                .find(|&(_, ref b)| b.covers(address))
                .unwrap();

            index = idx;
            split = !buckets_maxed && bucket.is_full() && bucket.covers(&self_address);
        }

        if split {
            let mut bucket = self.buckets.remove(index);
            let (a, b) = bucket.split();
            self.buckets.extend(vec![a, b]);
            self.bucket_for(address, func);
        } else {
            let ref mut bucket = self.buckets.get_mut(index).unwrap();
            func(bucket);
        }
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
        fn address(&self) -> Address {
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
}
