extern crate env_logger;
extern crate crypto;
#[macro_use]
extern crate log;
extern crate mio;
extern crate num;
extern crate protobuf;
extern crate rand;
extern crate rustc_serialize;
extern crate time;

use address::Address;
use node::UdpNode;
use std::env;
use std::io;
use std::sync::mpsc;
use std::thread;
use client::Task;
use client::messages::TextMessage;

mod address;
mod client;
mod messages;
mod network;
mod node;
mod node_bucket;
mod routing_table;
mod stun;
mod transaction;
#[cfg(test)]
mod tests;

fn main() {
    env_logger::init().unwrap();
    let args: Vec<String> = env::args().collect();
    let secret = args[1].clone();

    let address = Address::for_content(secret.as_str());
    let host = args[2].as_str();

    let routers: Vec<Box<node::Node>> = match args.get(3) {
        Some(router_address) => {
            let router_node = Box::new(UdpNode::new(Address::null(), router_address.as_str()));
            vec![router_node]
        }
        None => vec![]
    };

    let network = network::Network::new(address, host, routers);
    let mut client = client::Client::new(address);
    let (event_sender, events) = mpsc::channel();
    client.register_event_listener(event_sender);
    let client_channel = client.run(network);

    thread::spawn(move || {
        for event in events {
            println!("Event: {:?}", event);
        }
    });

    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        if line.len() <= 1 {
            client_channel.send(Task::Shutdown).unwrap();
        } else {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            let recipient = Address::from_str(parts[0]);
            let message_text = parts[1].trim().to_string();

            let text_message = TextMessage::new(address, message_text);
            client_channel
                .send(Task::ScheduleMessageDelivery(recipient, text_message))
                .unwrap_or_else(|err| info!("Couldn't schedule message delivery: {:?}", err));
        }
    }
}
