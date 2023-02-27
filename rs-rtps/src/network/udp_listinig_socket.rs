use socket2::{Socket, SockAddr, Domain, Type, Protocol};
use std::net::{Ipv4Addr, SocketAddr};
use mio::net::UdpSocket;
use std::io;
use std::str::FromStr;

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

pub fn new_multicast(addr: &str, port: u16) -> UdpSocket {
    // TODO: multicast groupが既に存在しているか確認
    // consder: multicast groupの指定方法
    let socket = new_listening_socket(addr, port, false).unwrap();
    socket.join_multicast_v4(&Ipv4Addr::from_str("239.255.0.1").unwrap(), &Ipv4Addr::UNSPECIFIED).unwrap();
    socket
}
