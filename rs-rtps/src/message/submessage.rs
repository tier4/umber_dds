pub mod element;
pub mod submessage_flag;
pub mod submessage_header;
use crate::message::submessage::submessage_header::*;
use bytes::Bytes;
use speedy::Readable;
use std::{fmt, fmt::Debug};

use crate::message::submessage::element::{
    acknack::AckNack, data::Data, datafrag::DataFrag, gap::Gap, heartbeat::Heartbeat,
    heartbeatfrag::HeartbeatFrag, infodst::InfoDestination, inforeply::InfoReply,
    inforeply_ip4::InfoReplyIp4, infosrc::InfoSource, infots::InfoTimestamp, nackfrag::NackFrag,
};
use crate::message::submessage::submessage_flag::*;
use enumflags2::BitFlags;

pub struct SubMessage {
    pub header: SubMessageHeader,
    pub body: SubMessageBody,
}

pub enum SubMessageBody {
    Entity(EntitySubmessage),
    Interpreter(InterpreterSubmessage),
}

#[allow(non_camel_case_types)]
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

pub enum InterpreterSubmessage {
    InfoSource(InfoSource, BitFlags<InfoSourceFlag>),
    InfoDestinatio(InfoDestination, BitFlags<InfoDestionationFlag>),
    InfoTImestamp(InfoTimestamp, BitFlags<InfoTimestampFlag>),
    InfoReply(InfoReply, BitFlags<InfoReplyFlag>),
    InfoReplyIp4(InfoReplyIp4, BitFlags<InfoReplyIp4Flag>),
}
