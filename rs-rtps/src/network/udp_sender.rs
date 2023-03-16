use socket2::{Socket, SockAddr, Domain, Type, Protocol};
use std::net::{Ipv4Addr, SocketAddr};
use std::net::UdpSocket; // RustDDS use mio::net::UdpSocket here. I dont'n know why they don't use
                         // std::net::UdpSocket so, I use std::net::UdpSocket.
use std::io;

struct UdpSender {
    unicast_socket: UdpSocket,
    multicast_sockets: Vec<UdpSocket>,
}

impl UdpSender {

    fn new(sender_port: u16) -> io::Result<Self> {
        
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), sender_port);
        let unicast_socket = UdpSocket::bind(addr).unwrap();
        // TODO: open multicast sockets
        /* let raw_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        raw_socket.set_reuse_address(reuse_addr)?;

        let address = SocketAddr::new(addr.parse().unwrap(), port);

        raw_socket.bind(&SockAddr::from(address))?;

        let udp_socket = std::net::UdpSocket::from(raw_socket);
        udp_socket.set_nonblocking(true).expect("Clouldn't set nonbloking");
        let mio_socket = UdpSocket::from_std(udp_socket); */
        let mut multicast_sockets: Vec<UdpSocket> = Vec::new();
        Ok(Self {unicast_socket, multicast_sockets })
    }

}
