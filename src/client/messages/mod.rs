pub mod protobufs;

use address::Address;

#[derive(Debug, Clone)]
pub struct TextMessage {
    pub id: Address,
    pub sender: Address,
    pub text: String
}


impl TextMessage {
    pub fn new(sender: Address, text: String) -> TextMessage {
        // TODO: Better message ID generation (UUID?)
        let id = Address::for_content(format!("{}{}", sender.to_str(), text).as_str());
        TextMessage {
            id: id,
            sender: sender,
            text: text
        }
    }

    pub fn envelope(self, recipient: Address) -> Envelope {
        Envelope {
            recipient: recipient,
            message: Message::TextMessage(self)
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageAcknowledgement {
    pub message_id: Address
}

impl MessageAcknowledgement {
    pub fn new(message_id: Address) -> MessageAcknowledgement {
        MessageAcknowledgement {
            message_id: message_id
        }
    }

    pub fn envelope(self, recipient: Address) -> Envelope {
        Envelope {
            recipient: recipient,
            message: Message::MessageAcknowledgement(self)
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TextMessage(TextMessage),
    MessageAcknowledgement(MessageAcknowledgement)
}

#[derive(Debug, Clone)]
pub struct Envelope {
    pub recipient: Address,
    pub message: Message
}

impl Envelope {
    pub fn encode(self) -> Vec<u8> {
        use protobuf::Message as MessageForFunctions;
        let mut message = protobufs::Envelope::new();
        message.set_recipient(self.recipient.to_str());

        match self.message {
            Message::TextMessage(text_message) => {
                let mut encoded = protobufs::TextMessage::new();
                encoded.set_id(text_message.id.to_str());
                encoded.set_sender(text_message.sender.to_str());
                encoded.set_text(text_message.text);
                message.set_message_type(protobufs::Envelope_Type::TEXT_MESSAGE);
                message.set_text_message(encoded);
            }
            Message::MessageAcknowledgement(message_acknowledgement) => {
                let mut encoded = protobufs::MessageAcknowledgement::new();
                encoded.set_message_id(message_acknowledgement.message_id.to_str());
                message.set_message_type(protobufs::Envelope_Type::MESSAGE_ACKNOWLEDGEMENT);
                message.set_message_acknowledgement(encoded);
            }
        }

        message.write_to_bytes().unwrap()
    }
}

pub fn decode(data: Vec<u8>) -> Envelope {
    use protobuf;
    use std::io::Cursor;
    let mut data = Cursor::new(data);
    let envelope = protobuf::parse_from_reader::<protobufs::Envelope>(&mut data).unwrap();
    match envelope.get_message_type() {
        protobufs::Envelope_Type::TEXT_MESSAGE => {
            let message = envelope.get_text_message();
            Envelope {
                recipient: Address::from_str(envelope.get_recipient()),
                message: Message::TextMessage(TextMessage {
                    id: Address::from_str(message.get_id()),
                    sender: Address::from_str(message.get_sender()),
                    text: message.get_text().to_string()
                })
            }
        }
        protobufs::Envelope_Type::MESSAGE_ACKNOWLEDGEMENT => {
            let ack = envelope.get_message_acknowledgement();
            Envelope {
                recipient: Address::from_str(envelope.get_recipient()),
                message: Message::MessageAcknowledgement(MessageAcknowledgement {
                    message_id: Address::from_str(ack.get_message_id())
                })
            }
        }
    }
}
