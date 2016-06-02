pub mod protobufs;

use address::Address;
use mio;
use network;
use std::sync::mpsc;
use std::thread;
use std::io;
use protobuf::{self, Message};

#[derive(Debug)]
struct CommMessage {
    recipient: Address,
    sender: Address,
    text: String
}

impl CommMessage {
    fn new(recipient: Address, sender: Address, text: String) -> CommMessage {
        CommMessage {
            recipient: recipient,
            sender: sender,
            text: text
        }
    }

    fn decode(data: Vec<u8>) -> CommMessage {
        use std::io::Cursor;
        let mut data = Cursor::new(data);
        let message = protobuf::parse_from_reader::<protobufs::CommMessage>(&mut data).unwrap();
        CommMessage::new(
            Address::from_str(message.get_recipient()),
            Address::from_str(message.get_sender()),
            message.get_text().to_string())
    }

    fn encode(self) -> Vec<u8> {
        let mut message = protobufs::CommMessage::new();
        message.set_recipient(self.recipient.to_str());
        message.set_sender(self.sender.to_str());
        message.set_text(self.text);
        message.write_to_bytes().unwrap()
    }
}

#[derive(Debug)]
pub enum Task {
    HandleNetworkEvent(network::Event)
}

pub struct Client {
    address: Address
}

impl Client {
    pub fn new(address: Address) -> Client {
        Client {
            address: address
        }
    }

    pub fn run(mut self, mut network: network::Network) {
        let mut event_loop = mio::EventLoop::new().unwrap();
        let (event_sender, event_receiver) = mpsc::channel();
        network.register_event_listener(event_sender);
        let notify_channel = event_loop.channel();
        let sender = self.address;

        thread::spawn(move|| {
            for event in event_receiver.iter() {
                notify_channel.send(Task::HandleNetworkEvent(event)).unwrap();
            }
        });

        thread::spawn(move || event_loop.run(&mut self).unwrap());

        let task_sender = network.run();

        loop {
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            let recipient = Address::from_str(parts[0]);
            let message_text = parts[1].to_string();

            let message = CommMessage::new(recipient, sender, message_text);

            task_sender.send(network::OneshotTask::SendPacket(
                    recipient, message.encode())).unwrap();
        }
    }

    fn handle_networking_event(&mut self, event: network::Event, event_loop: &mut mio::EventLoop<Client>) {
        match event {
            network::Event::ReceivedPacket(data) => {
                let message = CommMessage::decode(data);
                if message.recipient == self.address {
                    println!("{}: {}", message.sender, message.text);
                } else {
                    println!("should forward: {:?}", message);
                }
            }
        }
    }
}

impl mio::Handler for Client {
    type Timeout = ();
    type Message = Task;

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Client>, task: Task) {
        match task {
            Task::HandleNetworkEvent(event) => self.handle_networking_event(event, event_loop)
        }
    }
}
