use address::Address;
use node::Node;
use protobuf;
use std::io::Read;

pub mod protobufs;

pub enum Query<'a> {
    FindNode(&'a Box<Node>, Address)
}

pub enum Response<'a> {
    FindNode(&'a Box<Node>, Vec<&'a Box<Node>>)
}

pub enum Message<'a> {
    Query(u32, Query<'a>),
    Response(u32, Response<'a>)
}

impl<'a> Message<'a> {
    fn parse_from_reader(reader: &mut Read) {
        let message = protobuf::parse_from_reader::<protobufs::Envelope>(reader).unwrap();
        match message.get_message_type() {
            protobufs::Envelope_Type::FIND_NODE_QUERY => {
            }
            _ => {}
        }
    }

    pub fn serialize(self) -> Vec<u8> {
        use protobuf::Message as Asdf;
        let mut envelope = protobufs::Envelope::new();
        match self {
            Message::Response(transaction_id, response) => {
                envelope.set_transaction_id(transaction_id);

                match response {
                    Response::FindNode(origin, nodes) => {
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
                    }
                }
            }
            Message::Query(transaction_id, query) => {
                envelope.set_transaction_id(transaction_id);

                match query {
                    Query::FindNode(origin, target) => {
                        envelope.set_message_type(protobufs::Envelope_Type::FIND_NODE_QUERY);
                        let mut query = protobufs::FindNodeQuery::new();
                        query.set_origin(origin.serialize());
                        query.set_target(target.to_str());
                        envelope.set_find_node_query(query);
                    }
                }
            }
        }
        envelope.write_to_bytes().unwrap()
    }
}
