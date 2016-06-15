use node_bucket;
use node_bucket::NodeBucket;
use address::{Address, Addressable, LENGTH};
use node::Node;
use transaction::TransactionIdGenerator;

#[derive(Debug, PartialEq)]
pub enum InsertOutcome {
    Ignored,    // Currently just for ignoring self-node
    Inserted,   // Inserted new node
    Updated,    // Updated existing node
    Discarded   // Bucket is full
}

pub type InsertionResult = Result<InsertOutcome, String>;

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

    // TODO: i don't like how much this function has to know about sending pings
    pub fn insert(&mut self, node: Box<Node>, self_node: &Box<Node>, transaction_ids:
                  &mut TransactionIdGenerator) -> InsertionResult {
        use messages::outgoing;

        if node.addresss() == self.self_address {
            return Ok(InsertOutcome::Ignored);
        }

        let index = self.bucket_for(&node.addresss());
        let mut bucket = self.buckets.remove(index);
        let self_address = self.self_address;

        if !self.buckets_maxed() && bucket.is_full() && bucket.covers(&self_address) {
            let (a, b) = bucket.split();
            self.buckets.insert(index, a);
            self.buckets.insert(index + 1, b);
            self.insert(node, self_node, transaction_ids)
        } else {
            let status = match bucket.insert(node) {
                Ok(node_bucket::InsertOutcome::Inserted) => Ok(InsertOutcome::Inserted),
                Ok(node_bucket::InsertOutcome::Updated) => Ok(InsertOutcome::Updated),
                Ok(node_bucket::InsertOutcome::Discarded) => {
                    for n in bucket.questionable_nodes() {
                        let transaction_id = transaction_ids.generate();
                        let query = outgoing::create_ping_query(
                            transaction_id, self_node);
                        n.send(query);
                        n.sent_query(transaction_id);
                    }
                    Ok(InsertOutcome::Discarded)
                }
                Err(error) => Err(error)
            };
            self.buckets.insert(index, bucket);
            status
        }
    }

    pub fn bucket_needing_refresh(&self) -> Option<&NodeBucket> {
        use rand::{thread_rng, Rng};
        let mut buckets: Vec<&NodeBucket>= self.buckets.iter().filter(|ref b| b.needs_refresh()).collect();
        if buckets.is_empty() {
            None
        } else {
            let len = buckets.len();
            let bucket = buckets.remove(thread_rng().gen_range(0, len));
            Some(bucket)
        }
    }

    pub fn find_node(&mut self, address: &Address) -> Option<&mut Box<Node>> {
        let index = self.bucket_for(address);
        let bucket = self.buckets.get_mut(index).unwrap();
        bucket.find_node(address)
    }

    pub fn nearest(&mut self) -> Vec<&mut Box<Node>> {
        let self_address = self.self_address;
        self.nearest_to(&self_address, true)
    }

    pub fn nearest_to(&mut self, address: &Address, include_routers: bool) -> Vec<&mut Box<Node>> {
        // TODO: this should walk buckets much more efficiently

        let mut candidates: Vec<&mut Box<Node>> = self.buckets
            .iter_mut()
            .flat_map(|b| b.get_nodes())
            .collect();

        candidates.sort_by_key(|n| n.addresss().distance_from(address));

        // chain on routers in case we don't have enough nodes yet
        if include_routers {
            candidates.into_iter().chain(self.routers.iter_mut()).take(self.k).collect()
        } else {
            candidates.into_iter().take(self.k).collect()
        }
    }

    pub fn questionable_nodes(&mut self) -> Vec<&mut Box<Node>> {
        // TODO: this should walk buckets much more efficiently

        self.buckets
            .iter_mut()
            .flat_map(|b| b.get_nodes())
            .filter(|n| n.is_questionable())
            .collect()
    }

    pub fn remove_bad_nodes(&mut self) {
        for bucket in self.buckets.iter_mut() {
            bucket.remove_bad_nodes();
        }
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
    use super::{InsertOutcome, RoutingTable};
    use node::Node;
    use tests::TestNode;
    use transaction::TransactionIdGenerator;

    #[test]
    fn test_insert() {
        let self_address = Address::from_str("0000000000000000000000000000000000000000");
        let self_node: Box<Node> = Box::new(TestNode::new(self_address));
        let mut transaction_ids = TransactionIdGenerator::new();
        let router = Box::new(TestNode::new(Address::null()));
        let mut table: RoutingTable = RoutingTable::new(2, self_address, vec![router]);
        let node_1 = Box::new(TestNode::new(Address::from_str("0000000000000000000000000000000000000001")));
        let node_2 = Box::new(TestNode::new(Address::from_str("ffffffffffffffffffffffffffffffffffffffff")));
        table.insert(node_1, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_2, &self_node, &mut transaction_ids).unwrap();
        assert_eq!(table.buckets.len(), 1);

        // Splits buckets upon adding a k+1th node in the same space as self node
        let node_3 = Box::new(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffe")));
        table.insert(node_3, &self_node, &mut transaction_ids).unwrap();
        assert_eq!(table.buckets.len(), 2);
        let node_4 = Box::new(TestNode::new(Address::from_str("7fffffffffffffffffffffffffffffffffffffff")));
        let node_5 = Box::new(TestNode::new(Address::from_str("7ffffffffffffffffffffffffffffffffffffffe")));
        table.insert(node_4, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_5, &self_node, &mut transaction_ids).unwrap();
        assert_eq!(table.buckets.len(), 3);

        // Replaces instead of duplicates existing nodes
        let node_6 = Box::new(TestNode::new(Address::from_str("0000000000000000000000000000000000000001")));
        let node_7 = Box::new(TestNode::new(Address::from_str("0000000000000000000000000000000000000001")));
        let node_8 = Box::new(TestNode::new(Address::from_str("0000000000000000000000000000000000000001")));
        table.insert(node_6, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_7, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_8, &self_node, &mut transaction_ids).unwrap();
        assert_eq!(table.buckets.len(), 3);

        // Disregards new nodes for full, non-self space buckets
        let node_9 = Box::new(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffd")));
        let node_10 = Box::new(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffc")));
        let node_11 = Box::new(TestNode::new(Address::from_str("fffffffffffffffffffffffffffffffffffffffb")));
        table.insert(node_9, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_10, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_11, &self_node, &mut transaction_ids).unwrap();
        assert_eq!(table.buckets.len(), 3);

        // Ignores self-node
        let node_12 = Box::new(TestNode::new(self_address));
        assert_eq!(table.insert(node_12, &self_node, &mut transaction_ids).unwrap(), InsertOutcome::Ignored);
        assert_eq!(table.buckets.len(), 3);
    }

    #[test]
    fn test_remove_bad_nodes() {
        let self_address = Address::from_str("0000000000000000000000000000000000000000");
        let self_node: Box<Node> = Box::new(TestNode::new(self_address));
        let mut transaction_ids = TransactionIdGenerator::new();
        let router = Box::new(TestNode::new(Address::null()));
        let mut table: RoutingTable = RoutingTable::new(8, self_address, vec![router]);
        let node_1 = Box::new(TestNode::new(Address::from_str("0000000000000000000000000000000000000001")));
        let node_2 = Box::new(TestNode::new(Address::from_str("0000000000000000000000000000000000000002")));
        let node_3 = Box::new(TestNode::new(Address::from_str("0000000000000000000000000000000000000003")));
        let node_4 = Box::new(TestNode::bad(Address::from_str("0000000000000000000000000000000000000004")));
        table.insert(node_1, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_2, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_3, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_4, &self_node, &mut transaction_ids).unwrap();
        assert!(table.buckets[0].contains(&Address::from_str("0000000000000000000000000000000000000004")));
        table.remove_bad_nodes();
        assert!(!table.buckets[0].contains(&Address::from_str("0000000000000000000000000000000000000004")));
    }

    #[test]
    fn test_nearest_to() {
        let self_address = Address::from_str("0000000000000000000000000000000000000000");
        let self_node: Box<Node> = Box::new(TestNode::new(self_address));
        let mut transaction_ids = TransactionIdGenerator::new();
        let router = Box::new(TestNode::new(Address::null()));
        let mut table: RoutingTable = RoutingTable::new(2, self_address, vec![router]);
        let addr_1 = Address::from_str("0000000000000000000000000000000000000001");
        let addr_2 = Address::from_str("7ffffffffffffffffffffffffffffffffffffffe");
        let addr_3 = Address::from_str("ffffffffffffffffffffffffffffffffffffffff");
        let node_1 = Box::new(TestNode::new(addr_1));
        let node_2 = Box::new(TestNode::new(addr_2));
        let node_3 = Box::new(TestNode::new(addr_3));
        table.insert(node_1, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_2, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_3, &self_node, &mut transaction_ids).unwrap();

        {
            let nearest = table.nearest_to(&Address::from_str("fffffffffffffffffffffffffffffffffffffffd"), false);
            assert_eq!(nearest[0].addresss(), addr_3);
            assert_eq!(nearest[1].addresss(), addr_2);
        }
        {
            let nearest = table.nearest_to(&Address::from_str("0000000000000000000000000000000000000002"), false);
            assert_eq!(nearest[0].addresss(), addr_1);
            assert_eq!(nearest[1].addresss(), addr_2);
        }
    }
}
