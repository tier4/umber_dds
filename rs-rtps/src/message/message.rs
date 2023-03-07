use bytes::BytesMut;
use speedy::{Readable, Writable};
use crate::message::submessage::SubMessage;

#[derive(Readable, Debug)]
struct RtpsVersion {
    major: u8,
    minor: u8,
}

#[derive(Readable, Debug)]
struct VenderId {
    vender_id: [u8; 2],
}

#[derive(Readable, Debug)]
struct GuidPrefix {
    guid_prefix: [u8; 12]
}

#[derive(Readable, Debug)]
pub struct Header {
    protocol: [u8; 4],
    version: RtpsVersion,
    venderId: VenderId,
    guidPrefix: u64,
}

pub struct Message {
    header: Header,
    submessages: Vec<SubMessage>,
}

impl Message {
    pub fn new(header: Header, submessages: Vec<SubMessage>) -> Message {
        Message { header, submessages }
    }

    pub fn handle_submessage(& self) {
        println!(">>>>>>>>>>>>>>>>");
        println!("header: {:?}", self.header);
        for submsg in &self.submessages {
            submsg.handle_submessage();
        }
        println!("<<<<<<<<<<<<<<<<\n");
    }
}
