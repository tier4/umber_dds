use crate::structure::{guid::*, vendorId::*};
use speedy::Readable;

#[derive(Readable, Debug)]
pub struct ProtocolVersion {
    major: u8,
    minor: u8,
}

impl ProtocolVersion {
    pub const PROTOCOLVERSION: Self = Self { major: 2, minor: 4 };
}

#[derive(Readable, Debug)]
pub struct ProtocolId {
    protocol_id: [u8; 4],
}

impl ProtocolId {
    pub const PROTOCOLVID: Self = Self {
        protocol_id: [b'R', b'T', b'P', b'S'],
    };
}

#[derive(Readable, Debug)]
pub struct Header {
    pub protocol: ProtocolId,
    pub version: ProtocolVersion,
    pub vendorId: VendorId,
    pub guidPrefix: GuidPrefix,
}
