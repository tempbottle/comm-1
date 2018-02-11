extern crate stun;
extern crate rand;

use rand::Rng;
use std::net::SocketAddr;

// TODO: These should be part of configuration
const STUN_SERVERS: &'static [&'static str] = &[
    "stun.l.google.com:19302",
    "stun1.l.google.com:19302",
    "stun2.l.google.com:19302",
    "stun3.l.google.com:19302",
    "stun4.l.google.com:19302"
];

pub fn get_mapped_address(local_addr: SocketAddr) -> Result<SocketAddr, String> {
    let stun_server = rand::thread_rng().choose(STUN_SERVERS).expect("No STUN server");
    let local_port = local_addr.port();
    let ip_version = match local_addr {
        SocketAddr::V4(_) => stun::IpVersion::V4,
        SocketAddr::V6(_) => stun::IpVersion::V6
    };
    let client = stun::Client::new(stun_server, local_port, ip_version);
    let mesage = stun::Message::request();
    let response = client.send(mesage.encode());
    let stun::Message { attributes, ..} = stun::Message::decode(response);

    if let stun::Attribute::XorMappedAddress(stun::XorMappedAddress(address)) = attributes[0] {
        Ok(address)
    } else {
        Err(format!("Couldn't resolve mapped adress"))
    }
}
