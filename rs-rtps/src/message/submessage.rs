pub mod submessage_flag;
pub mod submessage_header;
pub mod element;
use speedy::Readable;
use std::{fmt, fmt::Debug};
use bytes::Bytes;
use crate::message::submessage::submessage_header::*;

use enumflags2::BitFlags;
use crate::message::submessage::submessage_flag::*;
use crate::message::submessage::element::{acknack::AckNack, data::Data, datafrag::DataFrag, gap::Gap, heartbeat::Heartbeat, heartbeatfrag::HeartbeatFrag, nackfrag::NackFrag, infodst::InfoDestination, infosrc::InfoSource, inforeply::InfoReply, infots::InfoTimestamp};

pub struct SubMessage {
    pub header: SubMessageHeader,
    pub body: SubMessageBody,
}

impl SubMessage {

    pub fn new(header: SubMessageHeader, body_buf: Bytes) -> Option<SubMessage> {
        let body = match Self::parse_body_bud(&header, body_buf) {
            Some(b) => b,
            None => return None,
        };
        Some(SubMessage { header, body})
    }

    fn parse_body_bud(header: &SubMessageHeader, body_buf: Bytes) -> Option<SubMessageBody> {
        println!("submessage header: {:?}", header);
        print!("submessage {:?}: ", header.get_submessagekind());
        for i in body_buf.iter() {
            print!("0x{:02X} ", i);
        }
        println!();
        // TODO: body_buf をパースする
        // RustDDSでの該当箇所はsrc/dds/message_receiver.rs
        // let rtps_message = match Message::read_from_buffer(msg_bytes)
        // あたり
        // DATAとかの構造体にdeseriarizerをもたせてるっぽい
        let submessage_body = match header.get_submessagekind() {
            // entity
            SubMessageKind::DATA => {
                let f = BitFlags::<DataFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Entity(
                    EntitySubmessage::Data(Data::deserialize_data(&body_buf, f), f)
                )
            },
            /*
            SubMessageKind::DATA_FRAG => (),
            SubMessageKind::HEARTBEAT => (),
            SubMessageKind::HEARTBEAT_FRAG => (),
            SubMessageKind::GAP => (),
            SubMessageKind::ACKNACK => (),
            SubMessageKind::NACK_FRAG => (),
            // interpreter
            SubMessageKind::INFO_SRC => (),
            SubMessageKind::INFO_DST => (),
            SubMessageKind::INFO_TS => (),
            SubMessageKind::INFO_REPLY => (),
            SubMessageKind::INFO_REPLY_IP4 => (),
            */
            _ => return None,
        };
        Some(submessage_body)
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

pub enum EntitySubmessage {
    Data(Data, BitFlags<DataFlag>),
    DataFrag(DataFrag, BitFlags<DataFragFlag>),
    HeartBeat(Heartbeat, BitFlags<HeartbeatFlag>),
    HeartbeatFrag(HeartbeatFrag, BitFlags<HeartbeatFragFlag>),
    Gap(Gap, BitFlags<GapFlag>),
    AckNack(AckNack, BitFlags<AckNackFlag>),
    NackFrag(NackFrag, BitFlags<NackFragFlag>),
}

pub enum InterpreterSubmessage{
    InfoSource(InfoSource, InfoSourceFlag),
    InfoDestinatio(InfoDestination, InfoDestionationFlag),
    InfoTImestamp(InfoTimestamp, InfoTimestampFlag),
    InfoReply(InfoReply, InfoReplyFlag),
    Pad,
}
