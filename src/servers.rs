use std::sync::mpsc;
use std::net::{UdpSocket, SocketAddr};

use std::thread;
use network::{OneshotTask, TaskSender};
use node::{UdpTransport, Transport};
use stun;

pub struct UdpServer {
    host: SocketAddr
}

impl UdpServer {
    pub fn new(host: SocketAddr) -> UdpServer {
        UdpServer {
            host: host
        }
    }

    // TODO: I'd rather have some kind of select on two channels instead of this nested non-blocking
    // recv. It only works because of the 10ms timeout on the socket.
    fn run(&self, sender: TaskSender) -> mpsc::Sender<mpsc::Sender<()>> {
        use std::time::Duration;

        let (shutdown_sender, shutdown_receiver) = mpsc::channel::<mpsc::Sender<()>>();

        let host = self.host;

        thread::spawn(move || {
            let socket = UdpSocket::bind(host).unwrap();
            socket.set_read_timeout(Some(Duration::from_millis(10))).unwrap();

            loop {
                match shutdown_receiver.try_recv() {
                    Err(mpsc::TryRecvError::Empty) => {
                        let mut buf = [0; 4096];
                        match socket.recv(&mut buf) {
                            Ok(size) => {
                                sender.send(OneshotTask::Incoming(buf[..size].iter().cloned().collect()))
                                    .unwrap_or_else(|err| info!("Couldn't handling incoming: {:?}", err));
                            }
                            Err(_) => {
                                //warn!("Error receiving from server: {}", e)
                            }
                        }
                    }

                    Err(mpsc::TryRecvError::Disconnected) => {
                        // The other end of the shutdown_receiver was dropped
                        break
                    }

                    Ok(sentinel) => {
                        sentinel.send(()).unwrap();
                        break;
                    }
                }
            }
        });

        shutdown_sender
    }

    fn transport(&self) -> Transport {
        let mapped_host = stun::get_mapped_address(self.host).expect("Couldn't STUN myself");
        Transport::Udp(UdpTransport::new(mapped_host))
    }
}

pub enum Server {
    Udp(UdpServer)
}

impl Server {
    pub fn transport(&self) -> Transport {
        match self {
            Server::Udp(server) => server.transport()
        }
    }

    pub fn run(&self, sender: TaskSender) -> mpsc::Sender<mpsc::Sender<()>> {
        match self {
            Server::Udp(server) => server.run(sender)
        }
    }
}
