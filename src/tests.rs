use address::{Address, Addressable};
use messages;
use node::{Node, Serialize, Status};
use time;
use transaction::TransactionId;

#[derive(Debug)]
pub struct TestNode {
    pub address: Address,
    last_seen: time::Tm,
    status: Status
}

impl TestNode {
    pub fn new(address: Address) -> TestNode {
        TestNode {
            address: address,
            last_seen: time::empty_tm(),
            status: Status::Good
        }
    }

    pub  fn  questionable(address: Address) -> TestNode {
        TestNode {
            address: address,
            last_seen: time::empty_tm(),
            status: Status::Questionable
        }
    }

    pub fn bad(address: Address) -> TestNode {
        TestNode {
            address: address,
            last_seen: time::empty_tm(),
            status: Status::Bad
        }
    }
}

impl Addressable for TestNode {
    fn address(&self) -> Address {
        self.address
    }
}

impl Node for TestNode {
    fn is_bad(&self) -> bool {
        self.status == Status::Bad
    }

    fn is_good(&self) -> bool {
        self.status == Status::Good
    }

    fn is_questionable(&self) -> bool {
        self.status == Status::Questionable
    }

    fn last_seen(&self) -> time::Tm {
        self.last_seen
    }

    fn pending_query_count(&self) -> usize {
        0
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

