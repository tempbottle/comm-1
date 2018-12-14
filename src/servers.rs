use std::net::{ToSocketAddrs, SocketAddr};

use network::OneshotTask;
use node::{UdpTransport, Transport};
use stun;

pub struct UdpServer {
    socket_addr: SocketAddr,
    socket: Option<mio::udp::UdpSocket>
}

impl UdpServer {
    pub fn new(socket_addr: SocketAddr) -> UdpServer {
        UdpServer {
            socket_addr: socket_addr,
            socket: None
        }
    }

    fn run(&mut self) -> &mio::Evented {
        let socket = mio::udp::UdpSocket::bound(&self.socket_addr).expect("Couldn't bind socket");
        self.socket = Some(socket);
        return self.socket.as_ref().unwrap()
    }

    fn read(&self, channel: mio::Sender<OneshotTask>) {
        let mut buf = [0; 4096];
        let ref socket = self.socket.as_ref().expect("Must `run` the server before reading from it");
        if let Ok(Some((size, _))) = socket.recv_from(&mut buf) {
            channel
                .send(OneshotTask::Incoming(buf[..size].iter().cloned().collect()))
                .expect("Couldn't handle incoming");
        }
    }

    fn transport(&self) -> Transport {
        let mapped_host = stun::get_mapped_address(self.socket_addr).expect("Couldn't STUN myself");
        Transport::Udp(UdpTransport::new(mapped_host))
    }
}

pub enum Server {
    Udp(UdpServer)
}

impl Server {
    pub fn create(url: &str) -> Option<Server> {
        let parts: Vec<&str> = url.splitn(2, "://").collect();
        let protocol = parts[0];
        let host = parts[1];
        match protocol {
            "udp" => {
                if let Ok(mut socket_addrs) = host.to_socket_addrs() {
                    if let Some(socket_addr) = socket_addrs.next() {
                        return Some(Server::Udp(UdpServer::new(socket_addr)))
                    }
                }
                None
            }
            _ => None
        }
    }

    pub fn transport(&self) -> Transport {
        match self {
            Server::Udp(server) => server.transport()
        }
    }

    pub fn read(&self, channel: mio::Sender<OneshotTask>) {
        match self {
            Server::Udp(server) => server.read(channel)
        }
    }

    pub fn run(&mut self) -> &mio::Evented {
        match self {
            Server::Udp(server) => server.run()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Server;

    #[test]
    fn it_creates_udp_servers() {
        let server = Server::create("udp://0.0.0.0:6667");
        match server {
            Some(Server::Udp(_)) => assert!(true),
            _ => assert!(false)
        }
    }
}
