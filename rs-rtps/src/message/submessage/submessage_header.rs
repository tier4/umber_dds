use crate::message::submessage::*;
use speedy::Readable;

#[derive(Readable, Debug)]
pub struct SubMessageHeader {
    submessageId: u8,
    flags: u8,
    submessageLength: u16,
}

impl SubMessageHeader {
    pub fn get_content_len(&self) -> u16 {
        self.submessageLength
    }

    pub fn get_flags(&self) -> u8 {
        self.flags
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
