pub mod messages;

use address::Address;
use mio;
use network;
use self::messages::{Message, TextMessage, MessageAcknowledgement, Envelope};
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub enum Task {
    HandleNetworkEvent(network::Event),
    ScheduleMessageDelivery(Address, TextMessage)
}

#[derive(Debug)]
pub enum ScheduledTask {
    DeliverMessage(Address, TextMessage),
}

pub struct Client {
    address: Address,
    network_commands: Option<network::TaskSender>,
    received: HashSet<Address>,
    pending_deliveries: HashMap<Address, mio::Timeout>,
    delivered: HashMap<Address, usize>,
    acknowledgements: HashMap<Address, MessageAcknowledgement>
}

impl Client {
    pub fn new(address: Address) -> Client {
        Client {
            address: address,
            network_commands: None,
            received: HashSet::new(),
            pending_deliveries: HashMap::new(),
            delivered: HashMap::new(),
            acknowledgements: HashMap::new()
        }
    }

    pub fn run(mut self, mut network: network::Network, headless: bool) {
        let mut event_loop = mio::EventLoop::new().unwrap();
        let (event_sender, event_receiver) = mpsc::channel();
        network.register_event_listener(event_sender);
        let notify_channel = event_loop.channel();
        let sender = self.address;

        thread::spawn(move|| {
            for event in event_receiver.iter() {
                notify_channel
                    .send(Task::HandleNetworkEvent(event))
                    .unwrap_or_else(|err| info!("Couldn't handle network event: {:?}", err));
            }
        });

        self.network_commands = Some(network.run());

        let notify_channel = event_loop.channel();
        thread::spawn(move || event_loop.run(&mut self).unwrap());

        info!("Running client at {}", sender);

        if headless {
            loop { thread::park(); }
        } else {
            loop {
                let mut line = String::new();
                io::stdin().read_line(&mut line).unwrap();
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                let recipient = Address::from_str(parts[0]);
                let message_text = parts[1].trim().to_string();

                let text_message = TextMessage::new(sender, message_text);
                notify_channel
                    .send(Task::ScheduleMessageDelivery(recipient, text_message))
                    .unwrap_or_else(|err| info!("Couldn't schedule message delivery: {:?}", err));
            }
        }
    }

    fn handle_networking_event(&mut self, event: network::Event, event_loop: &mut mio::EventLoop<Client>) {
        match event {
            network::Event::ReceivedPacket(sender, data) => {
                let envelope = messages::decode(data);
                let Envelope { recipient, .. } = envelope;

                match envelope.message {
                    Message::TextMessage(text_message) => {
                        if recipient == self.address {
                            if self.received.insert(text_message.id) {
                                println!("{}: {}", text_message.sender, text_message.text);
                                let ack = MessageAcknowledgement::new(text_message.id);
                                self.deliver_acknowledgement(text_message.sender, ack, event_loop);
                            }
                        } else {
                            if let Some(ack) = self.acknowledgements.remove(&text_message.id) {
                                self.deliver_acknowledgement(sender, ack.clone(), event_loop);
                                self.acknowledgements.insert(text_message.id, ack);
                            } else {
                                self.schedule_message_delivery(recipient, text_message, event_loop);
                            }
                        }
                    }
                    Message::MessageAcknowledgement(ack) => {
                        self.pending_deliveries.remove(&ack.message_id).map(|p| event_loop.clear_timeout(p));

                        if let None = self.acknowledgements.insert(ack.message_id, ack.clone()) {
                            if recipient == self.address {
                                debug!("ack {}", ack.message_id);
                            } else {
                                self.deliver_acknowledgement(recipient, ack, event_loop);
                            }
                        }
                    }
                }
            }
        }
    }

    fn schedule_message_delivery(&mut self, recipient: Address, text_message: TextMessage, event_loop: &mut mio::EventLoop<Client>) {
        let message_id = text_message.id;
        if !self.pending_deliveries.contains_key(&message_id) {
            let delivered = self.delivered.entry(message_id).or_insert(0);
            let delay = (2u64.pow(*delivered as u32) - 1) * 1000;
            debug!("Deliver {} with delay {:?}", message_id, delay);
            let timeout = event_loop.timeout_ms(ScheduledTask::DeliverMessage(recipient, text_message), delay).unwrap();
            self.pending_deliveries.insert(message_id, timeout);
            *delivered += 1;
        }
    }

    fn deliver_acknowledgement(&mut self, recipient: Address, acknowledgement: MessageAcknowledgement, event_loop: &mut mio::EventLoop<Client>) {
        if let Some(ref commands) = self.network_commands {
            let envelope = acknowledgement.envelope(recipient);
            commands.send(network::OneshotTask::SendPacket(recipient, envelope.encode())).unwrap();
        }
    }

    fn deliver_message(&mut self, recipient: Address, text_message: TextMessage, event_loop: &mut mio::EventLoop<Client>) {
        let delivered = self.network_commands.as_ref().map(|commands| {
            let envelope = text_message.clone().envelope(recipient);
            commands.send(network::OneshotTask::SendPacket(recipient, envelope.encode()))
        }).unwrap();

        if let Ok(_) = delivered {
            self.pending_deliveries.remove(&text_message.id);
            self.schedule_message_delivery(recipient, text_message, event_loop);
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

    fn timeout(&mut self, event_loop: &mut mio::EventLoop<Client>, task: ScheduledTask) {
        match task {
            ScheduledTask::DeliverMessage(recipient, message) => self.deliver_message(recipient, message, event_loop)
        }
    }
}
