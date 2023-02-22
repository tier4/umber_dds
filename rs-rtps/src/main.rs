use std::net::{Ipv4Addr, UdpSocket};
mod network;
use network::{net_util, udp_listinig_socket};
mod dds;

fn main() {
    // multicast groupがすでに存在しているか確認
    let socket = udp_listinig_socket::new_listening_socket("0.0.0.0", 7400, true);
    loop {}
}
