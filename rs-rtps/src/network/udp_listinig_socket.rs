use socket2::{Socket, SockAddr, Domain, Type, Protocol};
use std::net::{Ipv4Addr, SocketAddr};
use mio::net::UdpSocket;
use std::io;

fn new_listening_socket(addr: &str, port: u16, reuse_addr: bool) -> io::Result<UdpSocket> {
    let raw_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    raw_socket.set_reuse_address(reuse_addr)?;

    let address = SocketAddr::new(addr.parse().unwrap(), port);

    raw_socket.bind(&SockAddr::from(address))?;

    let udp_socket = std::net::UdpSocket::from(raw_socket);
    let mio_socket = UdpSocket::from_std(udp_socket);
    Ok(mio_socket)
}

pub fn new_unicast(addr: &str, port: u16) -> UdpSocket {
    new_listening_socket(addr, port, false).unwrap()
}

pub fn new_multicast(addr: &str, port: u16, multicast_group: Ipv4Addr) -> UdpSocket {
    let socket = new_listening_socket(addr, port, false).unwrap();
    // TODO: multicast_group がmulticast adressか確認する
    socket.join_multicast_v4(&multicast_group, &Ipv4Addr::UNSPECIFIED).unwrap();
    socket
}
