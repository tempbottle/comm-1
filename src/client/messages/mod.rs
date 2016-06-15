pub mod protobufs;

use address::Address;

#[derive(Debug, Clone)]
pub struct CommMessage {
    pub recipient: Address,
    pub text_message: Option<TextMessage>,
    pub message_acknowledgement: Option<MessageAcknowledgement>
}

impl CommMessage {
    pub fn text_message(recipient: Address, sender: Address, text: String) -> CommMessage {
        let id = Address::for_content(format!(
                "{}{}{}", recipient.to_str(), sender.to_str(), text).as_str());
        CommMessage {
            recipient: recipient,
            text_message: Some(TextMessage {
                id: id,
                sender: sender,
                text: text
            }),
            message_acknowledgement: None
        }
    }

    pub fn message_acknowledgement(recipient: Address, message_id: Address) -> CommMessage {
        CommMessage {
            recipient: recipient,
            text_message: None,
            message_acknowledgement: Some(MessageAcknowledgement {
                id: message_id
            })
        }
    }

    pub fn encode(self) -> Vec<u8> {
        use protobuf::Message as MessageForFunctions;
        let mut message = protobufs::CommMessage::new();
        message.set_recipient(self.recipient.to_str());

        if let Some(text_message) = self.text_message {
            let mut encoded = protobufs::TextMessage::new();
            encoded.set_id(text_message.id.to_str());
            encoded.set_sender(text_message.sender.to_str());
            encoded.set_text(text_message.text);
            message.set_message_type(protobufs::CommMessage_Type::TEXT_MESSAGE);
            message.set_text_message(encoded);
        }

        if let Some(message_acknowledgement) = self.message_acknowledgement {
            let mut encoded = protobufs::MessageAcknowledgement::new();
            encoded.set_message_id(message_acknowledgement.id.to_str());
            message.set_message_type(protobufs::CommMessage_Type::MESSAGE_ACKNOWLEDGEMENT);
            message.set_message_acknowledgement(encoded);
        }

        message.write_to_bytes().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct TextMessage {
    pub id: Address,
    pub sender: Address,
    pub text: String
}

#[derive(Debug, Clone)]
pub struct MessageAcknowledgement {
    pub id: Address
}

pub fn decode(data: Vec<u8>) -> CommMessage {
    use protobuf;
    use std::io::Cursor;
    let mut data = Cursor::new(data);
    let comm_message = protobuf::parse_from_reader::<protobufs::CommMessage>(&mut data).unwrap();
    match comm_message.get_message_type() {
        protobufs::CommMessage_Type::TEXT_MESSAGE => {
            let message = comm_message.get_text_message();
            CommMessage {
                recipient: Address::from_str(comm_message.get_recipient()),
                text_message: Some(TextMessage {
                    id: Address::from_str(message.get_id()),
                    sender: Address::from_str(message.get_sender()),
                    text: message.get_text().to_string()
                }),
                message_acknowledgement: None
            }
        }
        protobufs::CommMessage_Type::MESSAGE_ACKNOWLEDGEMENT => {
            let ack = comm_message.get_message_acknowledgement();
            CommMessage {
                recipient: Address::from_str(comm_message.get_recipient()),
                text_message: None,
                message_acknowledgement: Some(MessageAcknowledgement {
                    id: Address::from_str(ack.get_message_id())
                })
            }
        }
    }
}
