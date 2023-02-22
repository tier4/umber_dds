
const PB: u16 = 7400;
const DG: u16 = 250;
const PG: u16 = 2;
const d0: u16 = 0;
const d1:u16 = 10;
const d2:u16 = 1;
const d3:u16 = 11;

pub fn spdp_multicast_port(domainId: u16) -> u16 {
    PB + DG * domainId + d0
}

pub fn spdp_unicast_port(domainId: u16, participantId: u16) -> u16 {
    PB + DG * domainId + d1 + PG * participantId
}

pub fn usertraffic_multicast_port(domainId: u16) -> u16 {
    PB + DG * domainId + d2
}

pub fn usertraffic_unicast_port(domainId: u16, participantId: u16) -> u16 {
    PB + DG * domainId + d3 + PG * participantId
}
