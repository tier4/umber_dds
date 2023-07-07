use crate::message::{
    submessage::{element::*, submessage_flag::*, submessage_header::*, *},
    *,
};
use crate::net_util::*;
use crate::structure::{guid::*, vendor_id::*};
use bytes::Bytes;
use speedy::Readable;
use text_colorizer::*;

pub struct MessageReceiver {
    own_guid_prefix: GuidPrefix,
    source_version: ProtocolVersion,
    source_vendor_id: VendorId,
    source_guid_prefix: GuidPrefix,
    dest_guid_prefix: GuidPrefix,
    unicast_reply_locator_list: Vec<Locator>,
    multicast_reply_locator_list: Vec<Locator>,
    have_timestamp: bool,
    timestamp: Timestamp,
}

impl MessageReceiver {
    pub fn new(participant_guidprefix: GuidPrefix) -> MessageReceiver {
        Self {
            own_guid_prefix: participant_guidprefix,
            source_version: ProtocolVersion::PROTOCOLVERSION,
            source_vendor_id: VendorId::VENDORID_UNKNOW,
            source_guid_prefix: GuidPrefix::UNKNOW,
            dest_guid_prefix: GuidPrefix::UNKNOW,
            unicast_reply_locator_list: vec![Locator::INVALID],
            multicast_reply_locator_list: vec![Locator::INVALID],
            have_timestamp: false,
            timestamp: Timestamp::TIME_INVALID,
        }
    }

    fn reset(&mut self) {
        self.source_version = ProtocolVersion::PROTOCOLVERSION;
        self.source_vendor_id = VendorId::VENDORID_UNKNOW;
        self.source_guid_prefix = GuidPrefix::UNKNOW;
        self.dest_guid_prefix = GuidPrefix::UNKNOW;
        self.unicast_reply_locator_list.clear();
        self.multicast_reply_locator_list.clear();
        self.have_timestamp = false;
        self.timestamp = Timestamp::TIME_INVALID;
    }

    pub fn handle_packet(&mut self, messages: Vec<UdpMessage>) {
        for message in messages {
            // Is DDSPING
            let msg = message.message;
            if msg.len() < 20 {
                if msg.len() >= 16 && msg[0..4] == b"RTPS"[..] && msg[9..16] == b"DDSPING"[..] {
                    println!("Received DDSPING");
                    return;
                }
            }
            let msg_buf = msg.freeze();
            let rtps_message = match Message::new(msg_buf) {
                Ok(m) => m,
                Err(e) => {
                    println!("{}: couldn't deserialize : {}", "error".red(), e);
                    return;
                }
            };
            self.handle_parsed_packet(rtps_message);
        }
    }

    fn handle_parsed_packet(&mut self, rtps_msg: Message) {
        self.reset();
        self.dest_guid_prefix = self.own_guid_prefix;
        self.source_guid_prefix = rtps_msg.header.guid_prefix;
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
                self.unicast_reply_locator_list = info_reply.unicast_locator_list;
                if flags.contains(InfoReplyFlag::Multicast) {
                    self.multicast_reply_locator_list = info_reply
                        .multicast_locator_list
                        .expect("invalid InfoReply");
                } else {
                    self.multicast_reply_locator_list.clear();
                }
            }
            InterpreterSubmessage::InfoReplyIp4(info_reply_ip4, flags) => {
                self.unicast_reply_locator_list = info_reply_ip4.unicast_locator_list;
                if flags.contains(InfoReplyIp4Flag::Multicast) {
                    self.multicast_reply_locator_list = info_reply_ip4
                        .multicast_locator_list
                        .expect("invalid InfoReplyIp4");
                } else {
                    self.multicast_reply_locator_list.clear();
                }
            }
            InterpreterSubmessage::InfoTImestamp(info_ts, flags) => {
                if !flags.contains(InfoTimestampFlag::Invalidate) {
                    self.have_timestamp = true;
                    self.timestamp = info_ts.timestamp.expect("invalid InfoTS");
                } else {
                    self.have_timestamp = false;
                }
            }
            InterpreterSubmessage::InfoSource(info_souce, _flags) => {
                self.source_guid_prefix = info_souce.guid_prefix;
                self.source_version = info_souce.protocol_version;
                self.source_vendor_id = info_souce.vendor_id;
                self.unicast_reply_locator_list.clear();
                self.multicast_reply_locator_list.clear();
                self.have_timestamp = false;
            }
            InterpreterSubmessage::InfoDestinatio(info_dst, _flags) => {
                if info_dst.guid_prefix != GuidPrefix::UNKNOW {
                    self.dest_guid_prefix = info_dst.guid_prefix;
                } else {
                    self.dest_guid_prefix = self.own_guid_prefix;
                }
            }
        }
    }
}
