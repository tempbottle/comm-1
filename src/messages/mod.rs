pub mod protobufs;

pub mod incoming {
    use std::io::Read;
    use address::Address;
    use node::{Deserialize, Node};
    use node;
    use protobuf;
    use super::protobufs;

    #[derive(Debug)]
    pub enum Query {
        FindNode(Box<Node>, Address)
    }

    #[derive(Debug)]
    pub enum Response {
        FindNode(Box<Node>, Vec<Box<Node>>)
    }

    #[derive(Debug)]
    pub enum Message {
        Query(u32, Query),
        Response(u32, Response)
    }

    pub fn parse_from_reader(reader: &mut Read) -> Result<Message, &str> {
        match protobuf::parse_from_reader::<protobufs::Envelope>(reader) {
            Ok(message) => {
                let transaction_id = message.get_transaction_id();
                match message.get_message_type() {
                    protobufs::Envelope_Type::FIND_NODE_QUERY => {
                        let find_node_query = message.get_find_node_query();
                        let origin = Box::new(node::UdpNode::deserialize(find_node_query.get_origin()));
                        let target = Address::from_str(find_node_query.get_target());
                        Ok(Message::Query(transaction_id, Query::FindNode(origin, target)))
                    }
                    protobufs::Envelope_Type::FIND_NODE_RESPONSE => {
                        let find_node_response = message.get_find_node_response();
                        let origin = Box::new(node::UdpNode::deserialize(find_node_response.get_origin()));
                        let nodes = find_node_response.get_nodes();
                        let nodes: Vec<Box<Node>> = nodes.iter().map(|n| {
                            let node: Box<Node> = Box::new(node::UdpNode::deserialize(n));
                            node
                        }).collect();
                        Ok(Message::Response(transaction_id, Response::FindNode(origin, nodes)))
                    }
                }
            }
            Err(_) => { Err("Failed to parse protobuf") }
        }
    }
}

pub mod outgoing {
    use address::Address;
    use node::Node;
    use protobuf;
    use super::protobufs;

    pub fn create_find_node_query(transaction_id: u32, origin: &Box<Node>, target: Address) -> Vec<u8> {
        use protobuf::Message;
        let mut envelope = protobufs::Envelope::new();
        envelope.set_transaction_id(transaction_id);
        envelope.set_message_type(protobufs::Envelope_Type::FIND_NODE_QUERY);
        let mut query = protobufs::FindNodeQuery::new();
        query.set_origin(origin.serialize());
        query.set_target(target.to_str());
        envelope.set_find_node_query(query);
        envelope.write_to_bytes().unwrap()
    }

    pub fn create_find_node_response(transaction_id: u32, origin: &Box<Node>, nodes: Vec<&Box<Node>>) -> Vec<u8> {
        use protobuf::Message;
        let mut envelope = protobufs::Envelope::new();
        envelope.set_transaction_id(transaction_id);
        envelope.set_message_type(protobufs::Envelope_Type::FIND_NODE_RESPONSE);
        let mut response = protobufs::FindNodeResponse::new();
        response.set_origin(origin.serialize());
        let nodes: Vec<protobufs::Node> = nodes
            .iter()
            .map(|n| n.serialize())
            .collect();
        response.set_nodes(protobuf::RepeatedField::from_slice(
                nodes.as_slice()));
        envelope.set_find_node_response(response);
        envelope.write_to_bytes().unwrap()
    }
}
