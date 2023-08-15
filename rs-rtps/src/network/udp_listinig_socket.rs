use crate::network::net_util::get_local_interfaces;
use mio_v06::net::UdpSocket;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn new_listening_socket(addr: &str, port: u16, reuse_addr: bool) -> io::Result<UdpSocket> {
    let raw_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    raw_socket.set_reuse_address(reuse_addr)?;

    let address = SocketAddr::new(addr.parse().unwrap(), port);

    raw_socket.bind(&SockAddr::from(address))?;

    let udp_socket = std::net::UdpSocket::from(raw_socket);
    udp_socket
        .set_nonblocking(true)
        .expect("Clouldn't set nonbloking");
    let mio_socket = UdpSocket::from_socket(udp_socket)?;
    Ok(mio_socket)
}

pub fn new_unicast(addr: &str, port: u16) -> UdpSocket {
    new_listening_socket(addr, port, false).unwrap()
}

pub fn new_multicast(addr: &str, port: u16, multicast_group: Ipv4Addr) -> UdpSocket {
    let socket = new_listening_socket(addr, port, false).unwrap();

    let local_interfaces = get_local_interfaces();
    for li in local_interfaces {
        match li {
            IpAddr::V4(a) => {
                socket.join_multicast_v4(&multicast_group, &a).unwrap();
            }
            IpAddr::V6(_) => continue,
        }
    }
    socket
}
