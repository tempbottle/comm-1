extern crate stun;

use std::net::SocketAddr;

pub fn get_mapped_address(local_addr: SocketAddr) -> Result<SocketAddr, String> {
    let local_port = local_addr.port();
    let client = stun::Client::new("stun.l.google.com:19302", local_port);
    let mesage = stun::Message::request();
    let response = client.send(mesage.encode());
    let stun::Message { attributes, ..} = stun::Message::decode(response);

    if let stun::Attribute::XorMappedAddress(stun::XorMappedAddress(address)) = attributes[0] {
        Ok(address)
    } else {
        Err(format!("Couldn't resolve mapped adress"))
    }
}
