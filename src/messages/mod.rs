pub mod protobufs;

pub mod incoming {
    use address::Address;
    use node::Node;
    use node;
    use protobuf;
    use std::io::Read;
    use super::protobufs;
    use transaction::TransactionId;

    #[derive(Debug)]
    pub enum Query {
        FindNode(Address),
        Packet(Vec<u8>),
        Ping
    }

    #[derive(Debug)]
    pub enum Response {
        FindNode(Vec<Box<Node>>),
        Packet,
        Ping
    }

    #[derive(Debug)]
    pub enum Message {
        Query(TransactionId, Box<Node>, Query),
        Response(TransactionId, Box<Node>, Response)
    }

    pub fn parse_from_reader(reader: &mut Read) -> Result<Message, &str> {
        match protobuf::parse_from_reader::<protobufs::Envelope>(reader) {
            Ok(message) => {
                let transaction_id = message.get_transaction_id();
                match message.get_message_type() {
                    protobufs::Envelope_Type::FIND_NODE_QUERY => {
                        let find_node_query = message.get_find_node_query();
                        let origin = node::deserialize(find_node_query.get_origin());
                        let target = Address::from_str(find_node_query.get_target());
                        Ok(Message::Query(transaction_id, origin, Query::FindNode(target)))
                    }
                    protobufs::Envelope_Type::FIND_NODE_RESPONSE => {
                        let find_node_response = message.get_find_node_response();
                        let origin = node::deserialize(find_node_response.get_origin());
                        let nodes = find_node_response.get_nodes();
                        let nodes: Vec<Box<Node>> = nodes.iter().map(|n| {
                            let node: Box<Node> = node::deserialize(n);
                            node
                        }).collect();
                        Ok(Message::Response(transaction_id, origin, Response::FindNode(nodes)))
                    }
                    protobufs::Envelope_Type::PING_QUERY => {
                        let ping_query = message.get_ping_query();
                        let origin = node::deserialize(ping_query.get_origin());
                        Ok(Message::Query(transaction_id, origin, Query::Ping))
                    },
                    protobufs::Envelope_Type::PING_RESPONSE => {
                        let ping_response = message.get_ping_response();
                        let origin = node::deserialize(ping_response.get_origin());
                        Ok(Message::Response(transaction_id, origin, Response::Ping))
                    },
                    protobufs::Envelope_Type::PACKET_QUERY => {
                        let packet_query = message.get_packet_query();
                        let origin = node::deserialize(packet_query.get_origin());
                        let payload = packet_query.get_payload();
                        Ok(Message::Query(transaction_id, origin, Query::Packet(payload.to_vec())))
                    },
                    protobufs::Envelope_Type::PACKET_RESPONSE => {
                        let response = message.get_packet_response();
                        let origin = node::deserialize(response.get_origin());
                        Ok(Message::Response(transaction_id, origin, Response::Packet))
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
    use protobuf::Message;
    use protobuf;
    use super::protobufs;
    use transaction::TransactionId;

    pub fn create_find_node_query(transaction_id: TransactionId, origin: &Box<Node>, target: &Address) -> Vec<u8> {
        let mut envelope = protobufs::Envelope::new();
        envelope.set_transaction_id(transaction_id);
        envelope.set_message_type(protobufs::Envelope_Type::FIND_NODE_QUERY);
        let mut query = protobufs::FindNodeQuery::new();
        query.set_origin(origin.serialize());
        query.set_target(target.to_str());
        envelope.set_find_node_query(query);
        envelope.write_to_bytes().unwrap()
    }

    pub fn create_find_node_response(transaction_id: TransactionId, origin: &Box<Node>, nodes: Vec<&mut Box<Node>>) -> Vec<u8> {
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

    pub fn create_ping_query(transaction_id: TransactionId, origin: &Box<Node>) -> Vec<u8> {
        let mut envelope = protobufs::Envelope::new();
        envelope.set_transaction_id(transaction_id);
        envelope.set_message_type(protobufs::Envelope_Type::PING_QUERY);
        let mut query = protobufs::PingQuery::new();
        query.set_origin(origin.serialize());
        envelope.set_ping_query(query);
        envelope.write_to_bytes().unwrap()
    }

    pub fn create_ping_response(transaction_id: TransactionId, origin: &Box<Node>) -> Vec<u8> {
        let mut envelope = protobufs::Envelope::new();
        envelope.set_transaction_id(transaction_id);
        envelope.set_message_type(protobufs::Envelope_Type::PING_RESPONSE);
        let mut response = protobufs::PingResponse::new();
        response.set_origin(origin.serialize());
        envelope.set_ping_response(response);
        envelope.write_to_bytes().unwrap()
    }

    pub fn create_packet_query(transaction_id: TransactionId, origin: &Box<Node>, payload: Vec<u8>) -> Vec<u8> {
        let mut envelope = protobufs::Envelope::new();
        envelope.set_transaction_id(transaction_id);
        envelope.set_message_type(protobufs::Envelope_Type::PACKET_QUERY);
        let mut query = protobufs::PacketQuery::new();
        query.set_origin(origin.serialize());
        query.set_payload(payload);
        envelope.set_packet_query(query);
        envelope.write_to_bytes().unwrap()
    }

    pub fn create_packet_response(transaction_id: TransactionId, origin: &Box<Node>) -> Vec<u8> {
        let mut envelope = protobufs::Envelope::new();
        envelope.set_transaction_id(transaction_id);
        envelope.set_message_type(protobufs::Envelope_Type::PACKET_RESPONSE);
        let mut response = protobufs::PacketResponse::new();
        response.set_origin(origin.serialize());
        envelope.set_packet_response(response);
        envelope.write_to_bytes().unwrap()
    }

}
