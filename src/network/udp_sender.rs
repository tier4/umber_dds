use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::UdpSocket;
use std::net::{IpAddr, Ipv4Addr, SocketAddr}; // RustDDS use mio::net::UdpSocket here. I dont'n know why they don't use
                                              // std::net::UdpSocket so, I use std::net::UdpSocket.
use crate::network::net_util;
use log::error;
use std::io;

pub struct UdpSender {
    unicast_socket: UdpSocket,
    multicast_sockets: Vec<UdpSocket>,
}

impl UdpSender {
    pub fn new(sender_port: u16) -> io::Result<Self> {
        // if 0.0.0.0 is binded to sender socket, source IP is decided automatic
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), sender_port);
        let unicast_socket = UdpSocket::bind(addr)
            .unwrap_or_else(|_| panic!("couldn't bind 0.0.0.0:{} to socket", sender_port));
        unicast_socket
            .set_multicast_loop_v4(true)
            .unwrap_or_else(|_| {
                panic!("couldn't set multicast_loop_v4 to 0.0.0.0:{}", sender_port)
            });

        let local_interfaces = net_util::get_local_interfaces();
        let mut multicast_sockets: Vec<UdpSocket> = Vec::new();
        // The DDS implementation sends multicast datagrams to all interfaces because it cannot determine which interfaces the nodes joined to the multicast group are connected to and which they are not.
        for li in local_interfaces {
            let raw_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
                .expect("couldn't open raw_socket");
            match li {
                IpAddr::V4(a) => {
                    raw_socket
                        .set_multicast_if_v4(&a)
                        .expect("couldn't set multicast_if_v4 to raw_socket");
                    let sockaddr = SockAddr::from(SocketAddr::new(li, 0));
                    raw_socket
                        .bind(&sockaddr)
                        .unwrap_or_else(|_| panic!("couldn't bind {:?} to raw_socket", sockaddr));
                }
                IpAddr::V6(_) => continue,
            }
            let mc_socket = std::net::UdpSocket::from(raw_socket);
            mc_socket
                .set_multicast_loop_v4(true)
                .expect("couldn't set multicast_loop_v4 to socket");
            multicast_sockets.push(mc_socket);
        }
        Ok(Self {
            unicast_socket,
            multicast_sockets,
        })
    }

    pub fn send_to_unicast(&self, data: &[u8], addr: Ipv4Addr, port: u16) {
        if let Err(e) = self.unicast_socket.send_to(data, (addr, port)) {
            error!("failed send data to {}:{} because '{:?}'", addr, port, e);
        }
    }

    pub fn send_to_multicast(&self, data: &[u8], multicast_group: Ipv4Addr, port: u16) {
        for msocket in &self.multicast_sockets {
            if let Err(e) = msocket.send_to(data, (multicast_group, port)) {
                error!(
                    "failed send data to {}:{} because '{:?}'",
                    multicast_group, port, e
                );
            }
        }
    }
}
