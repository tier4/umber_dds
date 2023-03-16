use crate::message::{*, submessage::{*, submessage_header::*, serialize::*, element::*}};
use crate::net_util::*;
use speedy::Readable;
use crate::structure::{guid::*, vendorId::*};
use bytes::Bytes;


pub struct MessageReceiver {
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
            sourceVersion: ProtocolVersion::PROTOCOLVERSION,
            sourceVendorId: VendorId::VENDORID_UNKNOW,
            sourceGuidPrefix: GuidPrefix::UNKNOW,
            destGuidPrefix: participant_guidprefix,
            unicastReplyLocatorList: vec!(Locator::INVALID),
            multicastReplyLocatorList: vec!(Locator::INVALID),
            haveTimestamp: false,
            timestamp: Timestamp::TIME_INVALID,
        }
    }

    pub fn handle_packet(&self, packets: Vec<UdpMessage>) {
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
            // ループの中で
            // bufの4byteをSubMessageHeaderにシリアライズ
            // bufの4からoctetsToNextHeaderをSubMessageBodyにシリアライズ
            // Vecに突っ込む
            let mut submessages: Vec<SubMessage> = Vec::new();
            while !rtps_body_buf.is_empty() {
                let submessage_header_buf = rtps_body_buf.split_to(4);
                // TODO: Message Receiverが従うルール (spec 8.3.4.1)に沿った実装に変更
                // 不正なsubmessageを受信した場合、残りのMessageは無視
                // Messageの拡張の実装
                let submessage_header = match SubMessageHeader::read_from_buffer(&submessage_header_buf) {
                    Ok(h) => h,
                    Err(_) => break,
                };
                let submessage_body_buf = rtps_body_buf.split_to(submessage_header.get_len() as usize);
                let submessage = SubMessage::new(submessage_header, submessage_body_buf);
                match submessage {
                    Some(msg) => submessages.push(msg),
                    None => println!("received UNKNOWN_RTPS or VENDORSPECIFIC"),
                }
            }
            let rtps_message = Message::new( rtps_header, submessages );
            self.handle_parsed_packet(rtps_message);
        }
    }

    fn handle_parsed_packet(&self, rtps_msg: Message){
        // TODO: 自身のメンバー変数をメッセージのヘッダに従って更新
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

    fn handle_entity_submessage(&self, e: EntitySubmessage) {
        println!("handle entity submsg");
        todo!();
    }
    fn handle_interpreter_submessage(&self, i: InterpreterSubmessage) {
        println!("handle entity submsg");
        todo!();
    }
}
