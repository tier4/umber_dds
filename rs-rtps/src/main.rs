use std::net::{Ipv4Addr, UdpSocket};
mod network;
use network::{net_util, udp_listinig_socket};
mod dds;
mod rtps;
use dds::{event_loop, participant::DomainParticipant};
mod message;
mod structure;

fn main() {
    let participant = DomainParticipant::new(0);
    loop {}
}
