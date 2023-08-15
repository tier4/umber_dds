use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::UdpSocket;
use std::net::{IpAddr, Ipv4Addr, SocketAddr}; // RustDDS use mio::net::UdpSocket here. I dont'n know why they don't use
                                              // std::net::UdpSocket so, I use std::net::UdpSocket.
use crate::network::net_util;
use std::io;

pub struct UdpSender {
    unicast_socket: UdpSocket,
    multicast_sockets: Vec<UdpSocket>,
}

impl UdpSender {
    pub fn new(sender_port: u16) -> io::Result<Self> {
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), sender_port);
        let unicast_socket = UdpSocket::bind(addr).unwrap();
        unicast_socket.set_multicast_loop_v4(true).unwrap();

        let local_interfaces = net_util::get_local_interfaces();
        let mut multicast_sockets: Vec<UdpSocket> = Vec::new();
        for li in local_interfaces {
            let raw_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
            match li {
                IpAddr::V4(a) => {
                    raw_socket.set_multicast_if_v4(&a).unwrap();
                    let sockaddr = SockAddr::from(SocketAddr::new(li, 0));
                    raw_socket.bind(&sockaddr).unwrap();
                }
                IpAddr::V6(_) => continue,
            }
            let mc_socket = std::net::UdpSocket::from(raw_socket);
            mc_socket.set_multicast_loop_v4(true).unwrap();
            multicast_sockets.push(mc_socket);
        }
        Ok(Self {
            unicast_socket,
            multicast_sockets,
        })
    }

    pub fn send_to_multicast(&self, data: &[u8], mutlicast_group: Ipv4Addr, port: u16) {
        for msocket in &self.multicast_sockets {
            match msocket.send_to(data, (mutlicast_group, port)) {
                Ok(_n) => (),
                Err(_) => panic!("udp send failed."),
            }
        }
    }
}
