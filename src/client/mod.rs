pub mod messages;

use address::Address;
use mio;
use network;
use self::messages::{CommMessage};
use std::collections::{HashMap, HashSet};
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
    pending_deliveries: HashMap<Address, mio::Timeout>,
    delivered: HashMap<Address, usize>,
    acknowledgements: HashMap<Address, CommMessage>
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

                let message = messages::create_text_message(recipient, sender, message_text);
                notify_channel
                    .send(Task::ScheduleMessageDelivery(recipient, message))
                    .unwrap_or_else(|err| info!("Couldn't schedule message delivery: {:?}", err));
            }
        }
    }

    fn handle_networking_event(&mut self, event: network::Event, event_loop: &mut mio::EventLoop<Client>) {
        match event {
            network::Event::ReceivedPacket(data) => {
                let comm_message = messages::decode(data);
                let CommMessage { recipient, .. } = comm_message.clone();

                if let Some(ref text_message) = comm_message.text_message {
                    if !self.received.contains(&text_message.id) {
                        self.received.insert(text_message.id);
                        if recipient == self.address {
                            println!("{}: {}", text_message.sender, text_message.text);
                            let ack = messages::create_message_acknowledgement(text_message.sender, text_message.id);
                            self.schedule_message_delivery(text_message.sender, ack, event_loop);
                        } else {
                            if let Some(ack) = self.acknowledgements.remove(&text_message.id) {
                                self.schedule_message_delivery(recipient, ack.clone(), event_loop);
                                self.acknowledgements.insert(text_message.id, ack);
                            } else {
                                self.schedule_message_delivery(recipient, comm_message.clone(), event_loop);
                            }
                        }
                    }
                }

                if let Some(ref ack) = comm_message.message_acknowledgement {
                    if let None = self.acknowledgements.insert(ack.id, comm_message.clone()) {
                        self.pending_deliveries.remove(&ack.id).map(|p| event_loop.clear_timeout(p));
                        if recipient == self.address {
                            debug!("ack {}", ack.id);
                        } else {
                            self.schedule_message_delivery(recipient, comm_message.clone(), event_loop);
                        }
                    }
                }
            }
        }
    }

    fn schedule_message_delivery(&mut self, recipient: Address, message: CommMessage, event_loop: &mut mio::EventLoop<Client>) {
        if let Some(ref text_message) = message.text_message {
            if !self.pending_deliveries.contains_key(&text_message.id) {
                let delivered = self.delivered.entry(text_message.id).or_insert(0);
                let delay = (2u64.pow(*delivered as u32) - 1) * 1000;
                debug!("Delivery with delay {:?}", delay);
                let timeout = event_loop.timeout_ms(ScheduledTask::DeliverMessage(recipient, message.clone()), delay).unwrap();
                self.pending_deliveries.insert(text_message.id, timeout);
                *delivered += 1;
            }
        }

        if let Some(_) = message.message_acknowledgement {
            event_loop.timeout_ms(ScheduledTask::DeliverMessage(recipient, message.clone()), 0).unwrap();
        }
    }

    fn deliver_message(&mut self, recipient: Address, message: CommMessage) {
        if let Some(ref commands) = self.network_commands {
            if let Some(ref text_message) = message.text_message {
                self.pending_deliveries.remove(&text_message.id);
            }
            commands.send(network::OneshotTask::SendPacket(recipient, message.encode())).unwrap();
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
