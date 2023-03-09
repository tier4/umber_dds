use speedy::Readable;
use crate::rtps::{submessage::SubMessage, vendorId::*, guid::*};

#[derive(Readable, Debug)]
pub struct ProtocolVersion {
    major: u8,
    minor: u8,
}

impl ProtocolVersion {
    pub const PROTOCOLVERSION: Self = Self {
        major: 2, minor: 4
    };
}


#[derive(Readable, Debug)]
struct ProtocolId {
    protocol_id: [u8; 4],
}

impl ProtocolId {
    pub const PROTOCOLVID: Self = Self {
        protocol_id: [b'R', b'T', b'P', b'S']
    };
}

#[derive(Readable, Debug)]
pub struct Header {
    protocol: ProtocolId,
    version: ProtocolVersion,
    vendorId: VendorId,
    guidPrefix: GuidPrefix,
}

pub struct Message {
    header: Header,
    submessages: Vec<SubMessage>,
}

impl Message {
    pub fn new(header: Header, submessages: Vec<SubMessage>) -> Message {
        Message { header, submessages }
    }

    pub fn handle_submessages(& self) {
        println!(">>>>>>>>>>>>>>>>");
        println!("header: {:?}", self.header);
        for submsg in &self.submessages {
            submsg.handle_submessage();
        }
        println!("<<<<<<<<<<<<<<<<\n");
    }
}
