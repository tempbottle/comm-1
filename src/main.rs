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
}
