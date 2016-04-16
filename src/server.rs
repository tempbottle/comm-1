use std::net::{SocketAddrV4, Ipv4Addr, UdpSocket};
use std::sync::mpsc;
use std::thread;

pub trait Server {
    fn start(&self, tx: mpsc::Sender<(usize, [u8; 4096])>);
}

pub struct UdpServer {
    port: u16
}

impl UdpServer {
    pub fn new(port: u16) -> UdpServer {
        UdpServer {
            port: port
        }
    }
}

impl Server for UdpServer {
    fn start(&self, tx: mpsc::Sender<(usize, [u8; 4096])>) {
        let ip = Ipv4Addr::new(0, 0, 0, 0);
        let address = SocketAddrV4::new(ip, self.port);
        let socket = UdpSocket::bind(address).unwrap();
        println!("Listening at {}", address);
        thread::spawn(move || {
            loop {
                let mut buf = [0; 4096];
                match socket.recv_from(&mut buf) {
                    Ok((size, _src)) => tx.send((size, buf)).unwrap(),
                    Err(e) => println!("Error: {}", e)
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use super::{Server, UdpServer};
    use std::net::UdpSocket;

    #[test]
    fn it_receives_udp_packets_sent_to_the_port() {
        let (tx, messages) = mpsc::channel();
        UdpServer::new(6666).start(tx);
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.send_to(&[1, 2, 3, 4], "0.0.0.0:6666").unwrap();
        let (size, data) = messages.recv().unwrap();
        assert_eq!(size, 4);
        assert_eq!(data[0], 1);
        assert_eq!(data[1], 2);
        assert_eq!(data[2], 3);
        assert_eq!(data[3], 4);
    }
}
