use mio_v06::net::UdpSocket;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io;
use std::net::{Ipv4Addr, SocketAddr};

fn new_listening_socket(addr: &str, port: u16, reuse_addr: bool) -> io::Result<UdpSocket> {
    let raw_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    raw_socket.set_reuse_address(reuse_addr)?;

    let address = SocketAddr::new(
        addr.parse()
            .unwrap_or_else(|_| panic!("could't parce '{addr}' to IP Address")),
        port,
    );

    raw_socket.bind(&SockAddr::from(address))?;

    let udp_socket = std::net::UdpSocket::from(raw_socket);
    udp_socket
        .set_nonblocking(true)
        .expect("could't set nonbloking");
    let mio_socket = UdpSocket::from_socket(udp_socket)?;
    Ok(mio_socket)
}

pub fn new_unicast(addr: &str, port: u16) -> io::Result<UdpSocket> {
    let sock = new_listening_socket(addr, port, false)?;
    Ok(sock)
}

pub fn new_multicast(
    addr: &str,
    port: u16,
    interfaces: &Vec<Ipv4Addr>,
    multicast_group: Ipv4Addr,
) -> io::Result<UdpSocket> {
    let socket = new_listening_socket(addr, port, true)?;

    for i in interfaces {
        socket.join_multicast_v4(&multicast_group, i)?;
    }
    Ok(socket)
}
