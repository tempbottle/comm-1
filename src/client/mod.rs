pub mod messages;

use address::Address;
use mio;
use network;
use self::messages::{Message, TextMessage, MessageAcknowledgement, Envelope};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::thread;

/// A command for the Client to execute immediately.
#[derive(Debug)]
pub enum Task {

    /// Handle events emitted by the `Network`, e.g. handling packets that we receive.
    HandleNetworkEvent(network::Event),

    /// Schedules a message to be delivered.
    ScheduleMessageDelivery(Address, TextMessage),

    /// Shuts down the `Client`. When it has completed the shutdown procedure, it will emit an
    /// `Event::Shutdown`.
    Shutdown
}

/// A task to be performed some time in the future
#[derive(Debug)]
pub enum ScheduledTask {

    /// Deliver a message to an address. This can be a message we are sending to someone else, or
    /// just a message we're relaying.
    DeliverMessage(Address, TextMessage),
}

/// Events emitted to any listeners registered with `register_event_listener`. They represent
/// various client-level events.
#[derive(Clone, Debug)]
pub enum Event {
    /// We've received a message that was addressed to us
    ReceivedTextMessage(TextMessage),

    /// We've sent or relayed a message
    SentTextMessage(TextMessage),

    /// We've received an acknowledgement for a message we sent
    ReceivedMessageAcknowledgement(MessageAcknowledgement),

    /// The client has shut down
    ///
    /// TODO: It might be useful for `Shutdown` to contain the serialized state of the entire
    /// `Client`, `Network`, `RoutingTable`, `Bucket`s and `Nodes` so that a user may store it and
    /// later re-initialize a `Client` with this state in order to prevent re-bootstrapping from
    /// scratch.
    Shutdown
}

/// A sender for issuing commands to the `Client`. Once a client is `run`, it is consumed and
/// methods cannot be called on it. A `TaskSender` as the asynchronous interface for controlling a
/// running `Client`.
pub type TaskSender = mio::Sender<Task>;

/// A receiver for receiving events from a running `Client`. Similar in purpose to `TaskSender`. A
/// running `Client` is consumed and cannot be queried via method calls. You must instead subscribe
/// to events it emits
pub type Events = mpsc::Receiver<Event>;

/// A `Client` handles receiving, processing, relaying, etc. of messages in the communication
/// network. It internalizes all the complex logic that is specific to messaging, and exposes a few
/// commands view a `TaskSender` and events via `Events`.
pub struct Client {
    address: Address,
    network_commands: Option<network::TaskSender>,
    received: HashSet<Address>,
    pending_deliveries: HashMap<Address, mio::Timeout>,
    delivered: HashMap<Address, usize>,
    acknowledgements: HashMap<Address, MessageAcknowledgement>,
    event_listeners: Vec<mpsc::Sender<Event>>
}

impl Client {
    /// Creates a new `Client`. `address` is the address of this client, i.e. where other clients
    /// should send messages intended for this client.
    pub fn new(address: Address) -> Client {
        Client {
            address: address,
            network_commands: None,
            received: HashSet::new(),
            pending_deliveries: HashMap::new(),
            delivered: HashMap::new(),
            acknowledgements: HashMap::new(),
            event_listeners: Vec::new()
        }
    }

    /// Starts a `Client` in its own thread and returns its task sender. This method consumes the
    /// `Client`, so all event listeners must be registered first.
    pub fn run(mut self, mut network: network::Network) -> TaskSender {
        let mut event_loop = mio::EventLoop::new().unwrap();
        let (event_sender, event_receiver) = mpsc::channel();
        network.register_event_listener(event_sender);
        let notify_channel = event_loop.channel();

        thread::spawn(move|| {
            for event in event_receiver.iter() {
                notify_channel
                    .send(Task::HandleNetworkEvent(event))
                    .unwrap_or_else(|err| info!("Couldn't handle network event: {:?}", err));
            }
        });

        self.network_commands = Some(network.run());

        let notify_channel = event_loop.channel();
        info!("Running client at {}", self.address);
        thread::spawn(move || event_loop.run(&mut self).unwrap());
        notify_channel
    }


    /// Registers an event listener that should be sent every `Event` the client emits.
    pub fn register_event_listener(&mut self, event_listener: mpsc::Sender<Event>) {
        self.event_listeners.push(event_listener);
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
                                let ack = MessageAcknowledgement::new(text_message.id);
                                let sender = text_message.sender;
                                let event = Event::ReceivedTextMessage(text_message);
                                self.broadcast_event(event);
                                self.deliver_acknowledgement(sender, ack, event_loop);
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
                        debug!("ack {}", ack.message_id);
                        self.pending_deliveries.remove(&ack.message_id).map(|p| event_loop.clear_timeout(p));

                        if let None = self.acknowledgements.insert(ack.message_id, ack.clone()) {
                            if recipient == self.address {
                                self.broadcast_event(Event::ReceivedMessageAcknowledgement(ack));
                            } else {
                                self.deliver_acknowledgement(recipient, ack, event_loop);
                            }
                        }
                    }
                }
            }
            network::Event::Shutdown => {
                event_loop.shutdown();
                self.broadcast_event(Event::Shutdown);
            }
        }
    }

    fn schedule_message_delivery(&mut self, recipient: Address, text_message: TextMessage, event_loop: &mut mio::EventLoop<Client>) {
        let message_id = text_message.id;
        if !self.pending_deliveries.contains_key(&message_id) {
            let delivered = self.delivered.entry(message_id).or_insert(0);
            let delay = (2u64.pow(*delivered as u32) - 1) * 1000;
            debug!("Deliver {} with delay {:?}", message_id, delay);
            let timeout = event_loop.timeout_ms(ScheduledTask::DeliverMessage(recipient, text_message.clone()), delay).unwrap();
            self.pending_deliveries.insert(message_id, timeout);
            *delivered += 1;
        }
        self.broadcast_event(Event::SentTextMessage(text_message));
    }

    fn deliver_acknowledgement(&mut self, recipient: Address, acknowledgement: MessageAcknowledgement, _event_loop: &mut mio::EventLoop<Client>) {
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

    fn broadcast_event(&self, event: Event) {
        for listener in self.event_listeners.iter() {
            listener.send(event.clone()).expect("Could not broadcast event");
        }
    }

    fn shutdown(&self, _event_loop: &mut mio::EventLoop<Client>) {
        if let Some(ref commands) = self.network_commands {
            commands.send(network::OneshotTask::Shutdown).unwrap();
        }
    }
}

impl mio::Handler for Client {
    type Timeout = ScheduledTask;
    type Message = Task;

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Client>, task: Task) {
        match task {
            Task::HandleNetworkEvent(event) => self.handle_networking_event(event, event_loop),
            Task::ScheduleMessageDelivery(recipient, message) => self.schedule_message_delivery(recipient, message, event_loop),
            Task::Shutdown => self.shutdown(event_loop)
        }
    }

    fn timeout(&mut self, event_loop: &mut mio::EventLoop<Client>, task: ScheduledTask) {
        match task {
            ScheduledTask::DeliverMessage(recipient, message) => self.deliver_message(recipient, message, event_loop)
        }
    }
}
