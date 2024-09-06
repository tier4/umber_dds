use crate::structure::{guid::*, vendor_id::*};
use serde::Deserialize;
use speedy::{Readable, Writable};

#[derive(Readable, Writable, Debug, Clone, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl ProtocolVersion {
    pub const PROTOCOLVERSION: Self = Self { major: 2, minor: 4 };
}

#[derive(Readable, Writable, Debug)]
pub struct ProtocolId {
    protocol_id: [u8; 4],
}

impl ProtocolId {
    pub const PROTOCOLVID: Self = Self {
        protocol_id: [b'R', b'T', b'P', b'S'],
    };
}

#[derive(Readable, Writable, Debug)]
pub struct Header {
    pub protocol: ProtocolId,
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}

impl Header {
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        Self {
            protocol: ProtocolId::PROTOCOLVID,
            version: ProtocolVersion::PROTOCOLVERSION,
            vendor_id: VendorId::THIS_IMPLEMENTATION,
            guid_prefix,
        }
    }
}
