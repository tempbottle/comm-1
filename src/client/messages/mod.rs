pub mod protobufs;

use address::Address;

#[derive(Debug, Clone)]
pub struct CommMessage {
    pub recipient: Address,
    pub message: Message
}

impl CommMessage {
    pub fn encode(self) -> Vec<u8> {
        use protobuf::Message as MessageForFunctions;
        let mut message = protobufs::CommMessage::new();
        message.set_recipient(self.recipient.to_str());

        match self.message {
            Message::TextMessage { id, sender, text } => {
                let mut text_message = protobufs::TextMessage::new();
                text_message.set_id(id.to_str());
                text_message.set_sender(sender.to_str());
                text_message.set_text(text);
                message.set_message_type(protobufs::CommMessage_Type::TEXT_MESSAGE);
                message.set_text_message(text_message);
            }

            Message::MessageAcknowledgement { id } => {
                let mut ack = protobufs::MessageAcknowledgement::new();
                ack.set_message_id(id.to_str());
                message.set_message_type(protobufs::CommMessage_Type::MESSAGE_ACKNOWLEDGEMENT);
                message.set_message_acknowledgement(ack);
            }
        }

        message.write_to_bytes().unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TextMessage {
        id: Address,
        sender: Address,
        text: String
    },

    MessageAcknowledgement {
        id: Address
    }
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
                message: Message::TextMessage {
                    id: Address::from_str(message.get_id()),
                    sender: Address::from_str(message.get_sender()),
                    text: message.get_text().to_string()
                }
            }
        }
        protobufs::CommMessage_Type::MESSAGE_ACKNOWLEDGEMENT => {
            let ack = comm_message.get_message_acknowledgement();
            CommMessage {
                recipient: Address::from_str(comm_message.get_recipient()),
                message: Message::MessageAcknowledgement {
                    id: Address::from_str(ack.get_message_id())
                }
            }
        }
    }
}

pub fn create_text_message(recipient: Address, sender: Address, text: String) -> CommMessage {
    let id = Address::for_content(format!(
            "{}{}{}", recipient.to_str(), sender.to_str(), text).as_str());
    CommMessage {
        recipient: recipient,
        message: Message::TextMessage {
            id: id,
            sender: sender,
            text: text
        }
    }
}

pub fn create_message_acknowledgement(recipient: Address, message_id: Address) -> CommMessage {
    CommMessage {
        recipient: recipient,
        message: Message::MessageAcknowledgement {
            id: message_id
        }
    }
}
