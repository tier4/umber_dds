use std::net::{Ipv4Addr, UdpSocket};
mod network;
use network::{net_util, udp_listinig_socket};
mod dds;
use dds::{participant::DomainParticipant, event_loop};
mod rtps;

fn main() {
    let participant = DomainParticipant::new(0);
    loop {}
}
