use speedy::Readable;
use std::{fmt, fmt::Debug};
use bytes::Bytes;

pub struct SubMessage {
    pub header: SubMessageHeader,
    pub element: SubMessageElement,
}

impl SubMessage {

    pub fn new(header: SubMessageHeader, body_buf: Bytes) -> Option<SubMessage> {
         use SubMessageKind::*;
         match header.get_submessagekind() {
             DATA | DATA_FRAG | HEARTBEAT | HEARTBEAT_FRAG |
             GAP | ACKNACK | NACK_FRAG
                => Some(SubMessage { header, element: SubMessageElement::Entity(body_buf) }),
             INFO_SRC | INFO_DST | INFO_REPLY |
             INFO_REPLY_IP4 | INFO_TS | PAD
                => Some(SubMessage { header, element: SubMessageElement::Interpreter(body_buf) }),
             UNKNOWN_RTPS => None,
             VENDORSPECIFIC => None,
         }
    }

    pub fn handle_submessage(& self) {
        match &self.element {
            SubMessageElement::Entity(e) => self.handel_entity_submessage(e),
            SubMessageElement::Interpreter(i) => self.handel_interpreter_submessage(i),
        }
    }

    fn handel_entity_submessage(&self, e: &Bytes) {
        println!("handle entity submsg");
        println!("submessage header: {:?}", self.header);
        println!("submessage kind: {:?}", self.header.get_submessagekind());
        println!("submessage body: {:?}", e);
        todo!(); // TODO:
    }

    fn handel_interpreter_submessage(&self, i: &Bytes) {
        println!("handle interpreter submsg");
        println!("submessage header: {:?}", self.header);
        println!("submessage kind: {:?}", self.header.get_submessagekind());
        println!("submessage body: {:?}", i);
        todo!(); // TODO:
    }

}

pub enum SubMessageElement {
    Entity(Bytes),
    Interpreter(Bytes),
}

#[derive(Readable, Debug)]
pub struct SubMessageHeader {
    submessageId: u8,
    flags: u8,
    submessageLength: u16,
}

impl SubMessageHeader {
    pub fn get_len(&self) -> u16 {
        self.submessageLength
    }

    pub fn get_submessagekind(&self) -> SubMessageKind {
        match self.submessageId {
            0x01 => SubMessageKind::PAD,
            0x06 => SubMessageKind::ACKNACK,
            0x07 => SubMessageKind::HEARTBEAT,
            0x08 => SubMessageKind::GAP,
            0x09 => SubMessageKind::INFO_TS,
            0x0c => SubMessageKind::INFO_SRC,
            0x0d => SubMessageKind::INFO_REPLY_IP4,
            0x0e => SubMessageKind::INFO_DST,
            0x0f => SubMessageKind::INFO_REPLY,
            0x12 => SubMessageKind::NACK_FRAG,
            0x13 => SubMessageKind::HEARTBEAT_FRAG,
            0x15 => SubMessageKind::DATA,
            0x16 => SubMessageKind::DATA_FRAG,
            0x00..=0x7f => SubMessageKind::UNKNOWN_RTPS,
            0x80..=0xff => SubMessageKind::VENDORSPECIFIC,
        }
    }

    pub fn get_endiaan_flag(&self) -> bool {
        match self.flags & 0x01 {
            0 => false,
            1 => true,
            _ => panic!("result of (flags & 0x01) isn't 0 or 1"),
        }
    }

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
