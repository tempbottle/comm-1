use node_bucket;
use node_bucket::NodeBucket;
use address::{Addressable, Address, LENGTH};
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
    routers: Vec<Node>,
    buckets: Vec<NodeBucket>
}

impl RoutingTable {
    pub fn new(k: usize, self_address: Address, routers: Vec<Node>) -> RoutingTable {
        let bucket = NodeBucket::new(k);
        RoutingTable {
            k: k,
            self_address: self_address,
            routers: routers,
            buckets: vec![bucket]
        }
    }

    // TODO: i don't like how much this function has to know about sending pings
    pub fn insert(&mut self, node: Node, self_node: &Node, transaction_ids:
                  &mut TransactionIdGenerator) -> InsertionResult {
        use messages::outgoing;

        if node.address() == self.self_address {
            return Ok(InsertOutcome::Ignored);
        }

        let index = self.bucket_for(&node.address());
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
                        n.sent_query(transaction_id);
                        n.send(query);
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

    pub fn find_node(&mut self, address: &Address) -> Option<&mut Node> {
        let index = self.bucket_for(address);
        let bucket = self.buckets.get_mut(index).unwrap();
        bucket.find_node(address)
    }

    pub fn nearest(&mut self) -> Vec<&mut Node> {
        let self_address = self.self_address;
        self.nearest_live_nodes_to(&self_address, true)
    }

    pub fn nearest_live_nodes_to(&mut self, address: &Address, include_routers: bool) -> Vec<&mut Node> {
        // TODO: this should walk buckets much more efficiently

        let mut candidates: Vec<&mut Node> = self.buckets
            .iter_mut()
            .flat_map(|b| b.get_nodes())
            .filter(|n| !n.is_bad())
            .collect();

        candidates.sort_by_key(|n| n.address().distance_from(address));

        // chain on routers in case we don't have enough nodes yet
        if include_routers {
            candidates.into_iter().chain(self.routers.iter_mut()).take(self.k).collect()
        } else {
            candidates.into_iter().take(self.k).collect()
        }
    }

    pub fn questionable_nodes(&mut self) -> Vec<&mut Node> {
        // TODO: this should walk buckets much more efficiently

        self.buckets
            .iter_mut()
            .flat_map(|b| b.get_nodes())
            .filter(|n| n.is_questionable())
            .collect()
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
    use address::{Addressable, Address};
    use super::{InsertOutcome, RoutingTable};
    use node;
    use transaction::TransactionIdGenerator;

    #[test]
    fn test_insert() {
        let self_address = Address::from_str("0000000000000000000000000000000000000000").unwrap();
        let self_node: node::Node = node::tests::good(self_address);
        let mut transaction_ids = TransactionIdGenerator::new();
        let router = node::tests::good(Address::null());
        let mut table: RoutingTable = RoutingTable::new(2, self_address, vec![router]);
        let node_1 = node::tests::good(Address::from_str("0000000000000000000000000000000000000001").unwrap());
        let node_2 = node::tests::good(Address::from_str("ffffffffffffffffffffffffffffffffffffffff").unwrap());
        table.insert(node_1, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_2, &self_node, &mut transaction_ids).unwrap();
        assert_eq!(table.buckets.len(), 1);

        // Splits buckets upon adding a k+1th node in the same space as self node
        let node_3 = node::tests::good(Address::from_str("fffffffffffffffffffffffffffffffffffffffe").unwrap());
        table.insert(node_3, &self_node, &mut transaction_ids).unwrap();
        assert_eq!(table.buckets.len(), 2);
        let node_4 = node::tests::good(Address::from_str("7fffffffffffffffffffffffffffffffffffffff").unwrap());
        let node_5 = node::tests::good(Address::from_str("7ffffffffffffffffffffffffffffffffffffffe").unwrap());
        table.insert(node_4, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_5, &self_node, &mut transaction_ids).unwrap();
        assert_eq!(table.buckets.len(), 3);

        // Replaces instead of duplicates existing nodes
        let node_6 = node::tests::good(Address::from_str("0000000000000000000000000000000000000001").unwrap());
        let node_7 = node::tests::good(Address::from_str("0000000000000000000000000000000000000001").unwrap());
        let node_8 = node::tests::good(Address::from_str("0000000000000000000000000000000000000001").unwrap());
        table.insert(node_6, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_7, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_8, &self_node, &mut transaction_ids).unwrap();
        assert_eq!(table.buckets.len(), 3);

        // Disregards new nodes for full, non-self space buckets
        let node_9 = node::tests::good(Address::from_str("fffffffffffffffffffffffffffffffffffffffd").unwrap());
        let node_10 = node::tests::good(Address::from_str("fffffffffffffffffffffffffffffffffffffffc").unwrap());
        let node_11 = node::tests::good(Address::from_str("fffffffffffffffffffffffffffffffffffffffb").unwrap());
        table.insert(node_9, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_10, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_11, &self_node, &mut transaction_ids).unwrap();
        assert_eq!(table.buckets.len(), 3);

        // Ignores self-node
        let node_12 = node::tests::good(self_address);
        assert_eq!(table.insert(node_12, &self_node, &mut transaction_ids).unwrap(), InsertOutcome::Ignored);
        assert_eq!(table.buckets.len(), 3);
    }

    #[test]
    fn test_nearest_live_node_to() {
        let self_address = Address::from_str("0000000000000000000000000000000000000000").unwrap();
        let self_node: node::Node = node::tests::good(self_address);
        let mut transaction_ids = TransactionIdGenerator::new();
        let router = node::tests::good(Address::null());
        let mut table: RoutingTable = RoutingTable::new(2, self_address, vec![router]);
        let addr_1 = Address::from_str("0000000000000000000000000000000000000001").unwrap();
        let addr_2 = Address::from_str("7ffffffffffffffffffffffffffffffffffffffe").unwrap();
        let addr_3 = Address::from_str("ffffffffffffffffffffffffffffffffffffffff").unwrap();
        let node_1 = node::tests::good(addr_1);
        let node_2 = node::tests::good(addr_2);
        let node_3 = node::tests::good(addr_3);
        table.insert(node_1, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_2, &self_node, &mut transaction_ids).unwrap();
        table.insert(node_3, &self_node, &mut transaction_ids).unwrap();

        {
            let nearest = table.nearest_live_nodes_to(&Address::from_str("fffffffffffffffffffffffffffffffffffffffd").unwrap(), false);
            assert_eq!(nearest[0].address(), addr_3);
            assert_eq!(nearest[1].address(), addr_2);
        }
        {
            let nearest = table.nearest_live_nodes_to(&Address::from_str("0000000000000000000000000000000000000002").unwrap(), false);
            assert_eq!(nearest[0].address(), addr_1);
            assert_eq!(nearest[1].address(), addr_2);
        }
    }
}
