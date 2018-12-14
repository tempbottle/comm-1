extern crate env_logger;
extern crate clap;
extern crate crypto;
#[macro_use]
extern crate log;
extern crate mio;
extern crate num;
extern crate protobuf;
extern crate rand;
extern crate rustc_serialize;
extern crate time;

use std::collections::HashSet;
use std::io;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::thread;

use address::Address;
use client::Task;
use client::messages::TextMessage;

mod address;
mod client;
mod messages;
mod network;
mod node;
mod node_bucket;
mod routing_table;
mod servers;
mod stun;
mod transaction;

/// Starts a command line client.
///
/// Usage:
///
///     comm SECRET LOCAL_ADDR BOOTSTRAP_ADDR
///
/// Example:
///
///     comm alpha 0.0.0.0:6667 10.0.1.13
fn main() {
    env_logger::init().unwrap();

    let matches = clap::App::new("comm")
        .version("0.1.0")
        .author("Zac Stewart <zgstewart@gmail.com>")
        .arg(clap::Arg::with_name("secret")
             .long("secret")
             .value_name("SECRET")
             .required(true)
             .takes_value(true))
        .arg(clap::Arg::with_name("server")
             .long("server")
             .short("s")
             .value_name("URL")
             .takes_value(true)
             .required(true)
             .multiple(true))
        .arg(clap::Arg::with_name("router")
             .long("router")
             .short("r")
             .value_name("URL")
             .takes_value(true)
             .multiple(true))
        .get_matches();

    let secret = matches.value_of("server").expect("No secret");

    let address = Address::for_content(secret);

    let servers = matches
        .values_of("server")
        .expect("No servers")
        .map(|url| servers::Server::create(url).expect("Invalid server spec"))
        .collect();

    let routers: Vec<node::Node> = match matches.values_of("router") {
        Some(urls) => {
            urls.map(|url| {
                let mut transports = HashSet::new();
                transports.insert(node::Transport::Udp(node::UdpTransport::new(
                            url.to_socket_addrs().unwrap().next().unwrap())));
                node::Node::new(Address::null(), transports)
            }).collect()
        }
        None => vec![]
    };

    let network = network::Network::new(address, servers, routers);
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
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        match parts.len() {
            1 => {
                client_channel.send(Task::Shutdown).expect("Couldn't send Shutdown")
            }
            2 => {
                let recipient = Address::from_str(parts[0]).unwrap();
                let message_text = parts[1].trim().to_string();

                let text_message = TextMessage::new(address, message_text);
                client_channel
                    .send(Task::ScheduleMessageDelivery(recipient, text_message))
                    .unwrap_or_else(|err| info!("Couldn't schedule message delivery: {:?}", err));
            }
            _ => {}
        }
    }
}
