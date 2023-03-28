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
    infosrc::InfoSource, infots::InfoTimestamp, nackfrag::NackFrag,
};
use crate::message::submessage::submessage_flag::*;
use enumflags2::BitFlags;

pub struct SubMessage {
    pub header: SubMessageHeader,
    pub body: SubMessageBody,
}

impl SubMessage {
    pub fn new(header: SubMessageHeader, body_buf: Bytes) -> std::io::Result<SubMessage> {
        let body = Self::parse_body_bud(&header, body_buf)?;
        Ok(SubMessage { header, body })
    }

    fn parse_body_bud(
        header: &SubMessageHeader,
        body_buf: Bytes,
    ) -> std::io::Result<SubMessageBody> {
        println!("submessage header: {:?}", header);
        print!("submessage {:?}: ", header.get_submessagekind());
        for i in body_buf.iter() {
            print!("0x{:02X} ", i);
        }
        println!();
        // DATA, DataFragはdeseriarizeにflagがひつようだからdeserializerを自前で実装
        // それ以外はspeedyをつかってdeserialize
        let e = header.get_endian();
        let submessage_body = match header.get_submessagekind() {
            // entity
            SubMessageKind::DATA => {
                let flags = BitFlags::<DataFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Entity(EntitySubmessage::Data(
                    Data::deserialize_data(&body_buf, flags)?,
                    flags,
                ))
            }
            SubMessageKind::DATA_FRAG => {
                let flags = BitFlags::<DataFragFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Entity(EntitySubmessage::DataFrag(
                    DataFrag::deserialize(&body_buf, flags)?,
                    flags,
                ))
            }
            SubMessageKind::HEARTBEAT => {
                let flags = BitFlags::<HeartbeatFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Entity(EntitySubmessage::HeartBeat(
                    Heartbeat::read_from_buffer_with_ctx(e, &body_buf)?,
                    flags,
                ))
            }
            SubMessageKind::HEARTBEAT_FRAG => {
                let flags = BitFlags::<HeartbeatFragFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Entity(EntitySubmessage::HeartbeatFrag(
                    HeartbeatFrag::read_from_buffer_with_ctx(e, &body_buf)?,
                    flags,
                ))
            }
            SubMessageKind::GAP => {
                let flags = BitFlags::<GapFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Entity(EntitySubmessage::Gap(
                    Gap::read_from_buffer_with_ctx(e, &body_buf)?,
                    flags,
                ))
            }
            SubMessageKind::ACKNACK => {
                let flags = BitFlags::<AckNackFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Entity(EntitySubmessage::AckNack(
                    AckNack::read_from_buffer_with_ctx(e, &body_buf)?,
                    flags,
                ))
            }
            SubMessageKind::NACK_FRAG => {
                let flags = BitFlags::<NackFragFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Entity(EntitySubmessage::NackFrag(
                    NackFrag::read_from_buffer_with_ctx(e, &body_buf)?,
                    flags,
                ))
            }
            // interpreter
            SubMessageKind::INFO_SRC => {
                let flags = BitFlags::<InfoSourceFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Interpreter(InterpreterSubmessage::InfoSource(
                    InfoSource::read_from_buffer_with_ctx(e, &body_buf)?,
                    flags,
                ))
            }
            SubMessageKind::INFO_DST => {
                let flags =
                    BitFlags::<InfoDestionationFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Interpreter(InterpreterSubmessage::InfoDestinatio(
                    InfoDestination::read_from_buffer_with_ctx(e, &body_buf)?,
                    flags,
                ))
            }
            SubMessageKind::INFO_TS => {
                let flags = BitFlags::<InfoTimestampFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Interpreter(InterpreterSubmessage::InfoTImestamp(
                    InfoTimestamp::read_from_buffer_with_ctx(e, &body_buf)?,
                    flags,
                ))
            }
            SubMessageKind::INFO_REPLY => {
                let flags = BitFlags::<InfoReplyFlag>::from_bits_truncate(header.get_flags());
                SubMessageBody::Interpreter(InterpreterSubmessage::InfoReply(
                    InfoReply::read_from_buffer_with_ctx(e, &body_buf)?,
                    flags,
                ))
            }
            _ => todo!(),
        };
        Ok(submessage_body)
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

pub enum InterpreterSubmessage {
    InfoSource(InfoSource, BitFlags<InfoSourceFlag>),
    InfoDestinatio(InfoDestination, BitFlags<InfoDestionationFlag>),
    InfoTImestamp(InfoTimestamp, BitFlags<InfoTimestampFlag>),
    InfoReply(InfoReply, BitFlags<InfoReplyFlag>),
}
