use bytes::BytesMut;
use speedy::{Readable, Writable};
use crate::rtps::submessage::SubMessage;

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
    rtps: [u8; 4],
    version: RtpsVersion,
    venderId: VenderId,
    guidPrefix: u64,
}

pub struct Message {
    header: Header,
    submessages: Vec<SubMessage>,
}
