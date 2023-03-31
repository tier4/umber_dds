pub mod message_header;
pub mod message_receiver;
pub mod submessage;

use crate::message::submessage::element::{
    acknack::AckNack, data::Data, datafrag::DataFrag, gap::Gap, heartbeat::Heartbeat,
    heartbeatfrag::HeartbeatFrag, infodst::InfoDestination, inforeply::InfoReply,
    inforeplyIp4::InfoReplyIp4, infosrc::InfoSource, infots::InfoTimestamp, nackfrag::NackFrag,
};
use crate::message::{
    message_header::*,
    submessage::{element::*, submessage_flag::*, submessage_header::*, SubMessage, *},
};
use bytes::Bytes;
use enumflags2::BitFlags;
use speedy::Readable;

pub struct Message {
    pub header: Header,
    pub submessages: Vec<SubMessage>,
}

impl Message {
    pub fn new(rtps_msg_buf: Bytes) -> std::io::Result<Message> {
        let rtps_header_buf = rtps_msg_buf.slice(..20);
        let rtps_header = match Header::read_from_buffer(&rtps_header_buf) {
            Ok(h) => h,
            Err(e) => panic!("{:?}", e), // TODO:
        };
        let mut rtps_body_buf = rtps_msg_buf.slice(20..);
        let mut submessages: Vec<SubMessage> = Vec::new();
        let sub_header_lenght = 4;
        while !rtps_body_buf.is_empty() {
            let submessage_header_buf = rtps_body_buf.split_to(sub_header_lenght);
            // TODO: Message Receiverが従うルール (spec 8.3.4.1)に沿った実装に変更
            let submessage_header = match SubMessageHeader::read_from_buffer(&submessage_header_buf)
            {
                Ok(h) => h,
                Err(_) => break,
            };
            // RTPS spec 2.3, section 9.4.5.1.3
            let submessage_body_len = if submessage_header.get_content_len() == 0 {
                match submessage_header.get_submessagekind() {
                    SubMessageKind::PAD | SubMessageKind::INFO_TS => 0,
                    _ => rtps_body_buf.len(),
                }
            } else {
                submessage_header.get_content_len() as usize
            };

            let submessage_body_buf = rtps_body_buf.split_to(submessage_body_len);
            println!("submessage header: {:?}", submessage_header);
            print!("submessage {:?}: ", submessage_header.get_submessagekind());
            for i in submessage_body_buf.iter() {
                print!("0x{:02X} ", i);
            }
            println!();
            // DATA, DataFragはdeseriarizeにflagがひつようだからdeserializerを自前で実装
            // それ以外はspeedyをつかってdeserialize
            let e = submessage_header.get_endian();
            let submessage_body = match submessage_header.get_submessagekind() {
                // entity
                SubMessageKind::DATA => {
                    let flags =
                        BitFlags::<DataFlag>::from_bits_truncate(submessage_header.get_flags());
                    SubMessageBody::Entity(EntitySubmessage::Data(
                        Data::deserialize_data(&submessage_body_buf, flags)?,
                        flags,
                    ))
                }
                SubMessageKind::DATA_FRAG => {
                    let flags =
                        BitFlags::<DataFragFlag>::from_bits_truncate(submessage_header.get_flags());
                    SubMessageBody::Entity(EntitySubmessage::DataFrag(
                        DataFrag::deserialize(&submessage_body_buf, flags)?,
                        flags,
                    ))
                }
                SubMessageKind::HEARTBEAT => {
                    let flags = BitFlags::<HeartbeatFlag>::from_bits_truncate(
                        submessage_header.get_flags(),
                    );
                    SubMessageBody::Entity(EntitySubmessage::HeartBeat(
                        Heartbeat::read_from_buffer_with_ctx(e, &submessage_body_buf)?,
                        flags,
                    ))
                }
                SubMessageKind::HEARTBEAT_FRAG => {
                    let flags = BitFlags::<HeartbeatFragFlag>::from_bits_truncate(
                        submessage_header.get_flags(),
                    );
                    SubMessageBody::Entity(EntitySubmessage::HeartbeatFrag(
                        HeartbeatFrag::read_from_buffer_with_ctx(e, &submessage_body_buf)?,
                        flags,
                    ))
                }
                SubMessageKind::GAP => {
                    let flags =
                        BitFlags::<GapFlag>::from_bits_truncate(submessage_header.get_flags());
                    SubMessageBody::Entity(EntitySubmessage::Gap(
                        Gap::read_from_buffer_with_ctx(e, &submessage_body_buf)?,
                        flags,
                    ))
                }
                SubMessageKind::ACKNACK => {
                    let flags =
                        BitFlags::<AckNackFlag>::from_bits_truncate(submessage_header.get_flags());
                    SubMessageBody::Entity(EntitySubmessage::AckNack(
                        AckNack::read_from_buffer_with_ctx(e, &submessage_body_buf)?,
                        flags,
                    ))
                }
                SubMessageKind::NACK_FRAG => {
                    let flags =
                        BitFlags::<NackFragFlag>::from_bits_truncate(submessage_header.get_flags());
                    SubMessageBody::Entity(EntitySubmessage::NackFrag(
                        NackFrag::read_from_buffer_with_ctx(e, &submessage_body_buf)?,
                        flags,
                    ))
                }
                // interpreter
                SubMessageKind::INFO_SRC => {
                    let flags = BitFlags::<InfoSourceFlag>::from_bits_truncate(
                        submessage_header.get_flags(),
                    );
                    SubMessageBody::Interpreter(InterpreterSubmessage::InfoSource(
                        InfoSource::read_from_buffer_with_ctx(e, &submessage_body_buf)?,
                        flags,
                    ))
                }
                SubMessageKind::INFO_DST => {
                    let flags = BitFlags::<InfoDestionationFlag>::from_bits_truncate(
                        submessage_header.get_flags(),
                    );
                    SubMessageBody::Interpreter(InterpreterSubmessage::InfoDestinatio(
                        InfoDestination::read_from_buffer_with_ctx(e, &submessage_body_buf)?,
                        flags,
                    ))
                }
                SubMessageKind::INFO_TS => {
                    let flags = BitFlags::<InfoTimestampFlag>::from_bits_truncate(
                        submessage_header.get_flags(),
                    );
                    let ts = if flags.contains(InfoTimestampFlag::Invalidate) {
                        None
                    } else {
                        Some(Timestamp::read_from_buffer_with_ctx(
                            e,
                            &submessage_body_buf,
                        )?)
                    };
                    SubMessageBody::Interpreter(InterpreterSubmessage::InfoTImestamp(
                        InfoTimestamp { timestamp: ts },
                        flags,
                    ))
                }
                SubMessageKind::INFO_REPLY => {
                    let flags = BitFlags::<InfoReplyFlag>::from_bits_truncate(
                        submessage_header.get_flags(),
                    );
                    SubMessageBody::Interpreter(InterpreterSubmessage::InfoReply(
                        InfoReply::read_from_buffer_with_ctx(e, &submessage_body_buf)?,
                        flags,
                    ))
                }
                SubMessageKind::INFO_REPLY_IP4 => {
                    let flags = BitFlags::<InfoReplyIp4Flag>::from_bits_truncate(
                        submessage_header.get_flags(),
                    );
                    SubMessageBody::Interpreter(InterpreterSubmessage::InfoReplyIp4(
                        InfoReplyIp4::read_from_buffer_with_ctx(e, &submessage_body_buf)?,
                        flags,
                    ))
                }
                SubMessageKind::PAD => continue,
                SubMessageKind::UNKNOWN_RTPS => continue,
                SubMessageKind::VENDORSPECIFIC => continue,
            };
            submessages.push(SubMessage {
                header: submessage_header,
                body: submessage_body,
            });
        }
        Ok(Message {
            header: rtps_header,
            submessages,
        })
    }
}
