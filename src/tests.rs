use address::{Address, Addressable};
use node::{Node, Serialize};
use transaction::TransactionId;
use messages;

#[derive(Debug)]
pub struct TestNode {
    pub address: Address
}

impl TestNode {
    pub fn new(address: Address) -> TestNode {
        TestNode { address: address }
    }
}

impl Addressable for TestNode {
    fn get_address(&self) -> Address {
        self.address
    }
}

impl Node for TestNode {
    fn is_questionable(&self) -> bool { false }

    fn received_query(&mut self, _: TransactionId) { }

    fn received_response(&mut self, _: TransactionId) { }

    fn send(&self, _: Vec<u8>) { }

    fn sent_query(&mut self, _: TransactionId) { }
}

impl Serialize for TestNode {
    fn serialize(&self) -> messages::protobufs::Node {
        messages::protobufs::Node::new()
    }
}

