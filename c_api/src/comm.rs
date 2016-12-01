extern crate env_logger;
extern crate comm;

use comm::address::Address;
use comm::client;
use comm::client::{Client, Event, Task, TaskSender};
use comm::network::Network;
use comm::node::{Node, UdpNode};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::mpsc;
use std::thread;

#[no_mangle]
pub extern "C" fn comm_initialize() {
    env_logger::init().expect("Couldn't initialize logger");
}

#[no_mangle]
pub extern "C" fn comm_address_for_content(content: *const c_char) -> *const Address {
    let c_str: &CStr = unsafe { CStr::from_ptr(content) };
    let address = Address::for_content(c_str.to_str().unwrap());
    Box::into_raw(Box::new(address))
}

#[no_mangle]
pub extern "C" fn comm_address_from_str(string: *const c_char) -> *const Address {
    let c_str: &CStr = unsafe { CStr::from_ptr(string) };
    let address = Address::from_str(c_str.to_str().unwrap());
    Box::into_raw(Box::new(address))
}

#[no_mangle]
pub extern "C" fn comm_address_null() -> *const Address {
    Box::into_raw(Box::new(Address::null()))
}

#[no_mangle]
pub extern "C" fn comm_address_copy(address: *const Address) -> *const Address {
    let copy = unsafe { *address }.clone();
    Box::into_raw(Box::new(copy))
}

// TODO: impl API to clean up leaked strings
#[no_mangle]
pub extern "C" fn comm_address_to_str(address: *const Address) -> *const c_char {
    let string = unsafe { *address }.to_str();
    let string = CString::new(string).unwrap();
    string.into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn comm_address_destroy(address: *mut Address) {
    let _ = Box::from_raw(address);
}

#[no_mangle]
pub extern "C" fn comm_udp_node_new(
    address: *const Address, socket_address: *const c_char) -> *const UdpNode {
    let socket_address: &CStr = unsafe { CStr::from_ptr(socket_address) };
    let node = UdpNode::new(unsafe { *address }, socket_address.to_str().unwrap());
    Box::into_raw(Box::new(node))
}

#[no_mangle]
pub unsafe extern "C" fn comm_udp_node_destroy(udp_node: *mut UdpNode) {
    let _ = Box::from_raw(udp_node);
}

#[no_mangle]
pub extern "C" fn comm_network_new(
    self_address: *mut Address, host: *const c_char,
    routers: *mut *mut UdpNode, routers_count: usize) -> *mut Network {
    let self_address = unsafe { *self_address };
    let host: &CStr = unsafe { CStr::from_ptr(host) };
    let routers = unsafe { Vec::from_raw_parts(routers, routers_count, routers_count) };
    let routers: Vec<Box<Node>> = routers.into_iter().map(|r| unsafe {
        Box::from_raw(r) as Box<Node>
    }).collect();
    let network = Network::new(self_address, host.to_str().unwrap(), routers);
    Box::into_raw(Box::new(network))
}

#[no_mangle]
pub unsafe extern "C" fn comm_network_run(network: *mut Network) {
    let network = Box::from_raw(network);
    network.run();
}

#[no_mangle]
pub unsafe extern "C" fn comm_network_destroy(network: *mut Network) {
    let _ = Box::from_raw(network);
}

#[no_mangle]
pub extern "C" fn comm_text_message_new(sender: *mut Address, text: *const c_char) -> *const client::messages::TextMessage {
    let sender = unsafe { *Box::from_raw(sender) };
    let c_str: &CStr = unsafe { CStr::from_ptr(text) };
    let message_text = c_str.to_str().unwrap().to_string();

    Box::into_raw(Box::new(client::messages::TextMessage::new(sender, message_text)))
}

// TODO: impl API to clean up leaked strings
#[no_mangle]
pub extern "C" fn comm_text_message_text(text_message: *const client::messages::TextMessage) -> *const c_char {
    let text_message = unsafe { &*text_message };
    let string = CString::new(text_message.text.clone()).unwrap();
    string.into_raw()
}

#[no_mangle]
pub extern "C" fn comm_text_message_sender(text_message: *const client::messages::TextMessage) -> *const Address {
    let text_message = unsafe { &*text_message };
    Box::into_raw(Box::new(text_message.sender))
}

#[no_mangle]
pub unsafe extern "C" fn comm_client_new(address: *const Address) -> *mut Client {
    Box::into_raw(Box::new(Client::new(*address)))
}

#[no_mangle]
pub unsafe extern "C" fn comm_client_run(
    client: *mut Client, network: *mut Network) -> *mut TaskSender {
    let client = Box::from_raw(client);
    let network = *Box::from_raw(network);
    Box::into_raw(Box::new(client.run(network)))
}

#[no_mangle]
pub extern "C" fn comm_client_register_text_message_received_callback(
    client: *mut Client, callback: extern fn(*const client::messages::TextMessage) -> ()) {

    let client = unsafe { &mut *client };
    let (event_sender, events) = mpsc::channel();
    client.register_event_listener(event_sender);

    thread::spawn(move || {
        for event in events {
            if let Event::ReceivedTextMessage(text_message) = event {
                callback(Box::into_raw(Box::new(text_message)));
            }
        }
    });
}

#[no_mangle]
pub extern "C" fn comm_client_commands_send_text_message(
    commands: *mut TaskSender,
    recipient: *mut Address,
    text_message: *mut client::messages::TextMessage) {
    let commands = unsafe { &mut *commands };
    let recipient = unsafe { *Box::from_raw(recipient) };
    let text_message = unsafe { *Box::from_raw(text_message) };
    commands.send(Task::ScheduleMessageDelivery(recipient, text_message)).unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn comm_client_destroy(client: *mut Client) {
    let _ = Box::from_raw(client);
}
