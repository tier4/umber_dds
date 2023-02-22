use socket2::{Socket, SockAddr, Domain, Type, Protocol};
use std::net::{Ipv4Addr, UdpSocket, SocketAddr};
use std::io;

pub fn new_listening_socket(addr: &str, port: u16, reuse_addr: bool) -> io::Result<UdpSocket> {
    let raw_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    raw_socket.set_reuse_address(reuse_addr);

    let address = SocketAddr::new(addr.parse().unwrap(), port);

    let socket = raw_socket.bind(&SockAddr::from(address))?;

    let udp_socket = std::net::UdpSocket::from(raw_socket);
    Ok(udp_socket)
}
