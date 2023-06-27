use bytes::BytesMut;
use mio::Token;
use std::net::SocketAddr;

const PB: u16 = 7400;
const DG: u16 = 250;
const PG: u16 = 2;
const D0: u16 = 0;
const D1: u16 = 10;
const D2: u16 = 1;
const D3: u16 = 11;

pub const DISCOVERY_UNI_TOKEN: Token = Token(0);
pub const DISCOVERY_MUTI_TOKEN: Token = Token(1);

pub struct UdpMessage {
    pub message: BytesMut,
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
