use bytes::BytesMut;
use if_addrs::get_if_addrs;
use log::warn;
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};

const PB: u16 = 7400;
const DG: u16 = 250;
const PG: u16 = 2;
const D0: u16 = 0;
const D1: u16 = 10;
const D2: u16 = 1;
const D3: u16 = 11;

pub struct UdpMessage {
    pub message: BytesMut,
    #[allow(dead_code)]
    pub addr: SocketAddr,
}

pub fn spdp_multicast_port(domain_id: u16) -> u16 {
    PB + DG * domain_id + D0
}

pub fn spdp_unicast_port(domain_id: u16, participant_id: u16) -> u16 {
    PB + DG * domain_id + D1 + PG * participant_id
}

pub fn usertraffic_multicast_port(domain_id: u16) -> u16 {
    PB + DG * domain_id + D2
}

pub fn usertraffic_unicast_port(domain_id: u16, participant_id: u16) -> u16 {
    PB + DG * domain_id + D3 + PG * participant_id
}

pub fn get_local_interfaces() -> Vec<IpAddr> {
    match get_if_addrs() {
        Ok(local_interface) => {
            let local_addrs: Vec<IpAddr> = local_interface
                .iter()
                .filter(|s| !s.is_loopback())
                .map(|s| s.ip())
                .collect();
            local_addrs
        }
        Err(e) => {
            warn!(
                "failed get local interface address because {:?}, use only loopback (127.0.0.1)",
                e
            );
            vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))]
        }
    }
}
