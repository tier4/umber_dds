use crate::message::{
    submessage::{element::*, submessage_flag::*, submessage_header::*, *},
    *,
};
use crate::net_util::*;
use crate::structure::{guid::*, vendorId::*};
use bytes::Bytes;
use speedy::Readable;

pub struct MessageReceiver {
    own_guidPrefix: GuidPrefix,
    sourceVersion: ProtocolVersion,
    sourceVendorId: VendorId,
    sourceGuidPrefix: GuidPrefix,
    destGuidPrefix: GuidPrefix,
    unicastReplyLocatorList: Vec<Locator>,
    multicastReplyLocatorList: Vec<Locator>,
    haveTimestamp: bool,
    timestamp: Timestamp,
}

impl MessageReceiver {
    pub fn new(participant_guidprefix: GuidPrefix) -> MessageReceiver {
        Self {
            own_guidPrefix: participant_guidprefix,
            sourceVersion: ProtocolVersion::PROTOCOLVERSION,
            sourceVendorId: VendorId::VENDORID_UNKNOW,
            sourceGuidPrefix: GuidPrefix::UNKNOW,
            destGuidPrefix: GuidPrefix::UNKNOW,
            unicastReplyLocatorList: vec![Locator::INVALID],
            multicastReplyLocatorList: vec![Locator::INVALID],
            haveTimestamp: false,
            timestamp: Timestamp::TIME_INVALID,
        }
    }

    fn reset(&mut self) {
        self.sourceVersion = ProtocolVersion::PROTOCOLVERSION;
        self.sourceVendorId = VendorId::VENDORID_UNKNOW;
        self.sourceGuidPrefix = GuidPrefix::UNKNOW;
        self.destGuidPrefix = GuidPrefix::UNKNOW;
        self.unicastReplyLocatorList.clear();
        self.multicastReplyLocatorList.clear();
        self.haveTimestamp = false;
        self.timestamp = Timestamp::TIME_INVALID;
    }

    pub fn handle_packet(&mut self, packets: Vec<UdpMessage>) {
        for packet in packets {
            // Is DDSPING
            let msg = packet.message;
            if msg.len() < 20 {
                if msg.len() >= 16 && msg[0..4] == b"RTPS"[..] && msg[9..16] == b"DDSPING"[..] {
                    println!("Received DDSPING");
                    return;
                }
            }
            let packet_buf = msg.freeze();
            let rtps_header_buf = packet_buf.slice(..20);
            let rtps_header = match Header::read_from_buffer(&rtps_header_buf) {
                Ok(h) => h,
                Err(e) => panic!("{:?}", e),
            };
            let mut rtps_body_buf = packet_buf.slice(20..);
            let mut submessages: Vec<SubMessage> = Vec::new();
            let sub_header_lenght = 4;
            while !rtps_body_buf.is_empty() {
                let submessage_header_buf = rtps_body_buf.split_to(sub_header_lenght);
                // TODO: Message Receiverが従うルール (spec 8.3.4.1)に沿った実装に変更
                // 不正なsubmessageを受信した場合、残りのMessageは無視
                let submessage_header =
                    match SubMessageHeader::read_from_buffer(&submessage_header_buf) {
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
                let submessage = SubMessage::new(submessage_header, submessage_body_buf);
                match submessage {
                    Some(msg) => submessages.push(msg),
                    None => println!("received UNKNOWN_RTPS or VENDORSPECIFIC"),
                }
            }
            let rtps_message = Message::new(rtps_header, submessages);
            self.handle_parsed_packet(rtps_message);
        }
    }

    fn handle_parsed_packet(&mut self, rtps_msg: Message) {
        self.reset();
        self.destGuidPrefix = self.own_guidPrefix;
        self.sourceGuidPrefix = rtps_msg.header.guidPrefix;
        println!(">>>>>>>>>>>>>>>>");
        println!("header: {:?}", rtps_msg.header);
        for submsg in rtps_msg.submessages {
            println!("submessage header: {:?}", submsg.header);
            println!("submessage kind: {:?}", submsg.header.get_submessagekind());
            match submsg.body {
                SubMessageBody::Entity(e) => self.handle_entity_submessage(e),
                SubMessageBody::Interpreter(i) => self.handle_interpreter_submessage(i),
            }
        }
        println!("<<<<<<<<<<<<<<<<\n");
    }

    fn handle_entity_submessage(&self, entity_subm: EntitySubmessage) {
        println!("handle entity submsg");
        todo!();
        match entity_subm {
            EntitySubmessage::AckNack(acknack, flags) => (),
            EntitySubmessage::Data(data, flags) => (),
            EntitySubmessage::DataFrag(dat_frag, flags) => (),
            EntitySubmessage::Gap(gap, flags) => (),
            EntitySubmessage::HeartBeat(heartbeat, flags) => (),
            EntitySubmessage::HeartbeatFrag(heartbeatfrag, flags) => (),
            EntitySubmessage::NackFrag(nack_frag, flags) => (),
        }
    }
    fn handle_interpreter_submessage(&mut self, interpreter_subm: InterpreterSubmessage) {
        println!("handle interpreter submsg");
        match interpreter_subm {
            InterpreterSubmessage::InfoReply(info_reply, flags) => {
                self.unicastReplyLocatorList = info_reply.unicastLocatorList;
                if flags.contains(InfoReplyFlag::Multicast) {
                    self.multicastReplyLocatorList = info_reply.multicastLocatorList;
                } else {
                    self.multicastReplyLocatorList.clear();
                }
            }
            InterpreterSubmessage::InfoTImestamp(info_ts, flags) => {
                if !flags.contains(InfoTimestampFlag::Invalidate) {
                    self.haveTimestamp = true;
                    self.timestamp = info_ts.timestamp;
                } else {
                    self.haveTimestamp = false;
                }
            }
            InterpreterSubmessage::InfoSource(info_souce, _flags) => {
                self.sourceGuidPrefix = info_souce.guidPrefix;
                self.sourceVersion = info_souce.protocolVersion;
                self.sourceVendorId = info_souce.vendorId;
                self.unicastReplyLocatorList.clear();
                self.multicastReplyLocatorList.clear();
                self.haveTimestamp = false;
            }
            InterpreterSubmessage::InfoDestinatio(info_dst, _flags) => {
                if info_dst.guidPrefix != GuidPrefix::UNKNOW {
                    self.destGuidPrefix = info_dst.guidPrefix;
                } else {
                    self.destGuidPrefix = self.own_guidPrefix;
                }
            }
        }
    }
}