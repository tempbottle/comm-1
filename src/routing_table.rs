use node_bucket::NodeBucket;
use address::{Address, Addressable, LENGTH};
use node::Node;

#[derive(Debug)]
pub struct RoutingTable {
    k: usize,
    self_address: Address,
    routers: Vec<Box<Node>>,
    buckets: Vec<NodeBucket>
}

impl RoutingTable {
    pub fn new(k: usize, self_address: Address, routers: Vec<Box<Node>>) -> RoutingTable {
        let bucket = NodeBucket::new(k);
        RoutingTable {
            k: k,
            self_address: self_address,
            routers: routers,
            buckets: vec![bucket]
        }
    }

    pub fn insert(&mut self, node: Box<Node>) {
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

    pub fn nearest_to(&self, address: &Address) -> Vec<&Box<Node>> {
        // TODO: this should walk buckets much more efficiently

        let mut candidates: Vec<&Box<Node>> = self.buckets
            .iter()
            .flat_map(|b| b.get_nodes())
            .collect();

        candidates.sort_by_key(|n| n.get_address().distance_from(address));

        // chain on routers in case we don't have enough nodes yet
        candidates.into_iter().chain(self.routers.iter()).take(self.k).collect()
    }

    fn bucket_for(&self, address: &Address) -> usize {
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
    use address::{Address, Addressable};
    use messages;
    use node::{Node, Serialize};
    use super::RoutingTable;

    #[derive(Debug)]
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

    impl Node for TestNode {
        fn update(&mut self) { }
        fn send(&self, _: Vec<u8>) { }
    }

    impl Serialize for TestNode {
        fn serialize(&self) -> messages::protobufs::Node {
            messages::protobufs::Node::new()
        }
    }

    #[test]
    fn test_insert() {
        let self_node = Address::from_str("0000000000000000000000000000000000000000");
        let router = Box::new(TestNode::new(Address::null()));
        let mut table: RoutingTable = RoutingTable::new(2, self_node, vec![router]);
        table.insert(Box::new(TestNode::new(Address::from_str("0000000000000000000000000000000000000001"))));
        table.insert(Box::new(TestNode::new(Address::from_str("ffffffffffffffffffffffffffffffffffffffff"))));
        assert_eq!(table.buckets.len(), 1);

        // Splits buckets upon adding a k+1th node in the same space as self node
        table.insert(Box::new(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffe"))));
        assert_eq!(table.buckets.len(), 2);
        table.insert(Box::new(TestNode::new(Address::from_str("7fffffffffffffffffffffffffffffffffffffff"))));
        table.insert(Box::new(TestNode::new(Address::from_str("7ffffffffffffffffffffffffffffffffffffffe"))));
        assert_eq!(table.buckets.len(), 3);

        // Replaces instead of duplicates existing nodes
        table.insert(Box::new(TestNode::new(Address::from_str("0000000000000000000000000000000000000001"))));
        table.insert(Box::new(TestNode::new(Address::from_str("0000000000000000000000000000000000000001"))));
        table.insert(Box::new(TestNode::new(Address::from_str("0000000000000000000000000000000000000001"))));
        assert_eq!(table.buckets.len(), 3);

        // Disregards new nodes for full, non-self space buckets
        table.insert(Box::new(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffd"))));
        table.insert(Box::new(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffc"))));
        table.insert(Box::new(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffb"))));
        assert_eq!(table.buckets.len(), 3);
    }

    #[test]
    fn test_nearest_to() {
        let self_node = Address::from_str("0000000000000000000000000000000000000000");
        let router = Box::new(TestNode::new(Address::null()));
        let mut table: RoutingTable = RoutingTable::new(2, self_node, vec![router]);
        let addr_1 = Address::from_str("0000000000000000000000000000000000000001");
        let addr_2 = Address::from_str("7ffffffffffffffffffffffffffffffffffffffe");
        let addr_3 = Address::from_str("ffffffffffffffffffffffffffffffffffffffff");
        let node_1 = Box::new(TestNode::new(addr_1));
        let node_2 = Box::new(TestNode::new(addr_2));
        let node_3 = Box::new(TestNode::new(addr_3));
        table.insert(node_1);
        table.insert(node_2);
        table.insert(node_3);

        let nearest = table.nearest_to(&Address::from_str("fffffffffffffffffffffffffffffffffffffffd"));
        assert_eq!(nearest[0].get_address(), addr_3);
        assert_eq!(nearest[1].get_address(), addr_2);
        let nearest = table.nearest_to(&Address::from_str("0000000000000000000000000000000000000002"));
        assert_eq!(nearest[0].get_address(), addr_1);
        assert_eq!(nearest[1].get_address(), addr_2);
    }
}
