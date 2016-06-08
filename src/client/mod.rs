pub mod messages;

use address::Address;
use mio;
use network;
use self::messages::{CommMessage, Message};
use std::collections::HashSet;
use std::io;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub enum Task {
    HandleNetworkEvent(network::Event),
    ScheduleMessageDelivery(Address, CommMessage)
}

#[derive(Debug)]
pub enum ScheduledTask {
    DeliverMessage(Address, CommMessage),
}

pub struct Client {
    address: Address,
    network_commands: Option<network::TaskSender>,
    received: HashSet<Address>,
    acknowledged: HashSet<Address>
}

impl Client {
    pub fn new(address: Address) -> Client {
        Client {
            address: address,
            network_commands: None,
            received: HashSet::new(),
            acknowledged: HashSet::new()
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

        self.network_commands = Some(network.run());

        let notify_channel = event_loop.channel();
        thread::spawn(move || event_loop.run(&mut self).unwrap());

        println!("Running client at {}", sender);

        loop {
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            let recipient = Address::from_str(parts[0]);
            let message_text = parts[1].trim().to_string();

            let message = messages::create_text_message(recipient, sender, message_text);
            notify_channel.send(Task::ScheduleMessageDelivery(recipient, message)).unwrap();
        }
    }

    fn handle_networking_event(&mut self, event: network::Event, event_loop: &mut mio::EventLoop<Client>) {
        match event {
            network::Event::ReceivedPacket(data) => {
                let comm_message = messages::decode(data);
                let CommMessage { recipient, message } = comm_message.clone();
                match message {
                    Message::TextMessage { id, sender, text } => {
                        if recipient == self.address && !self.received.contains(&id) {
                            println!("{}: {}", sender, text);
                            self.received.insert(id);
                            let ack = messages::create_message_acknowledgement(sender, id);
                            self.schedule_message_delivery(sender, ack, event_loop);
                        } else {
                            if !self.acknowledged.contains(&id) {
                                self.schedule_message_delivery(recipient, comm_message, event_loop);
                            }
                        }
                    }

                    Message::MessageAcknowledgement { id } => {
                        if self.acknowledged.insert(id) {
                            if recipient == self.address {
                                println!("ack {}", id);
                            } else {
                                self.schedule_message_delivery(recipient, comm_message, event_loop);
                            }
                        }
                    }
                }
            }
        }
    }

    fn schedule_message_delivery(&mut self, recipient: Address, message: CommMessage, event_loop: &mut mio::EventLoop<Client>) {
        event_loop.timeout_ms(ScheduledTask::DeliverMessage(recipient, message), 0).unwrap();
    }

    fn deliver_message(&mut self, recipient: Address, message: CommMessage) {
        match self.network_commands {
            Some(ref c) => c.send(network::OneshotTask::SendPacket(recipient, message.encode())).unwrap(),
            None => { println!("No network_commands") }
        }
    }
}

impl mio::Handler for Client {
    type Timeout = ScheduledTask;
    type Message = Task;

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Client>, task: Task) {
        match task {
            Task::HandleNetworkEvent(event) => self.handle_networking_event(event, event_loop),
            Task::ScheduleMessageDelivery(recipient, message) => self.schedule_message_delivery(recipient, message, event_loop)
        }
    }

    fn timeout(&mut self, _event_loop: &mut mio::EventLoop<Client>, task: ScheduledTask) {
        match task {
            ScheduledTask::DeliverMessage(recipient, message) => self.deliver_message(recipient, message)
        }
    }
}
