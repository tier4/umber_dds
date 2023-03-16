pub mod submessage_flag;
pub mod submessage_header;
pub mod serialize;
pub mod element;
use speedy::Readable;
use std::{fmt, fmt::Debug};
use bytes::Bytes;
use crate::message::submessage::{submessage_header::*, serialize::*};

pub struct SubMessage {
    pub header: SubMessageHeader,
    pub body: SubMessageBody,
}

impl SubMessage {

    pub fn new(header: SubMessageHeader, body_buf: Bytes) -> Option<SubMessage> {
        let body = match Self::parse_body_bud(body_buf) {
            Some(b) => b,
            None => return None,
        };
        Some(SubMessage { header, body})


    }

    fn parse_body_bud(body_buf: Bytes) -> Option<SubMessageBody> {
        // TODO: body_buf をパースする
        todo!();
    }
}

pub enum SubMessageBody {
    Entity(EntitySubmessage),
    Interpreter(InterpreterSubmessage),
}

pub enum SubMessageKind {
    PAD = 0x01,
    ACKNACK = 0x06,
    HEARTBEAT = 0x07,
    GAP = 0x08,
    INFO_TS = 0x09,
    INFO_SRC = 0x0c,
    INFO_REPLY_IP4 = 0x0d,
    INFO_DST = 0x0e,
    INFO_REPLY = 0x0f,
    NACK_FRAG = 0x12,
    HEARTBEAT_FRAG = 0x13,
    DATA = 0x15,
    DATA_FRAG = 0x16,
    UNKNOWN_RTPS,
    VENDORSPECIFIC,
}

impl Debug for SubMessageKind {

    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::PAD => fmt.write_str("PAD"),
            Self::ACKNACK => fmt.write_str("ACKNACK"),
            Self::HEARTBEAT => fmt.write_str("HEARTBEAT"),
            Self::GAP => fmt.write_str("GAP"),
            Self::INFO_TS => fmt.write_str("INFO_TS"),
            Self::INFO_SRC => fmt.write_str("INFO_SRC"),
            Self::INFO_REPLY_IP4 => fmt.write_str("INFO_REPLY_IP4"),
            Self::INFO_DST => fmt.write_str("INFO_DST"),
            Self::INFO_REPLY => fmt.write_str("INFO_REPLY"),
            Self::NACK_FRAG => fmt.write_str("NACK_FRAG"),
            Self::HEARTBEAT_FRAG => fmt.write_str("HEARTBEAT_FRAG"),
            Self::DATA => fmt.write_str("DATA"),
            Self::DATA_FRAG => fmt.write_str("DATA_FRAG"),
            Self::UNKNOWN_RTPS => fmt.write_str("UNKNOWN_RTPS"),
            Self::VENDORSPECIFIC => fmt.write_str("VENDORSPECIFIC"),
        }
    }
}

