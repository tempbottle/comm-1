use address::{Address, Addressable};
use messages;
use node::{Node, Serialize};
use time;
use transaction::TransactionId;

#[derive(Debug)]
pub struct TestNode {
    pub address: Address,
    last_seen: time::Tm
}

impl TestNode {
    pub fn new(address: Address) -> TestNode {
        TestNode {
            address: address,
            last_seen: time::empty_tm()
        }
    }
}

impl Addressable for TestNode {
    fn get_address(&self) -> Address {
        self.address
    }
}

impl Node for TestNode {

    fn is_questionable(&self) -> bool { false }

    fn last_seen(&self) -> time::Tm {
        self.last_seen
    }

    fn received_query(&mut self, _: TransactionId) {
        self.last_seen = time::now_utc();
    }

    fn received_response(&mut self, _: TransactionId) {
        self.last_seen = time::now_utc();
    }

    fn send(&self, _: Vec<u8>) { }

    fn sent_query(&mut self, _: TransactionId) { }
}

impl Serialize for TestNode {
    fn serialize(&self) -> messages::protobufs::Node {
        messages::protobufs::Node::new()
    }
}

