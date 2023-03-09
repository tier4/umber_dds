use crate::rtps::{message::*, submessage::*, vendorId::*};
use crate::net_util::*;
use speedy::Readable;
use crate::rtps::guid::*;


pub struct TimeStamp {
    seconds: u32,
    fractions: u32,
}

impl TimeStamp {
    pub const TIME_INVALID: Self = Self {
        seconds: 0x00,
        fractions: 0x00,
    };
}

// spec versin 2.3 9.3.2 Mapping of the Types that Appear Within Submessages or Built-in Topic Data
struct LocatorList {
    kind: i64,
    port: u64,
    address: [u8; 16],
}

impl LocatorList {
    pub const KIND_INVALID: i64 = -1;
    pub const KIND_RESERVED: i64 = 0;
    pub const KIND_UDPv4: i64 = 1;
    pub const KIND_UDPv6: i64 = 2;
    pub const PORT_INVALID: u64 = 0;
    pub const ADDRESS_INVALID: [u8; 16] = [0; 16];
    pub const INVALID: Self = Self { kind: Self::KIND_INVALID, port: Self::PORT_INVALID, address: Self::ADDRESS_INVALID };

}

pub struct MessageReceiver {
    sourceVersion: ProtocolVersion,
    sourceVendorId: VendorId,
    sourceGuidPrefix: GuidPrefix,
    destGuidPrefix: GuidPrefix,
    unicastReplyLocatorList: Vec<LocatorList>,
    multicastReplyLocatorList: Vec<LocatorList>,
    haveTimestamp: bool,
    timestamp: TimeStamp,
}

impl MessageReceiver {
    pub fn new(participant_guidprefix: GuidPrefix) -> MessageReceiver {
        Self {
            sourceVersion: ProtocolVersion::PROTOCOLVERSION,
            sourceVendorId: VendorId::VENDORID_UNKNOW,
            sourceGuidPrefix: GuidPrefix::UNKNOW,
            destGuidPrefix: participant_guidprefix,
            unicastReplyLocatorList: vec!(LocatorList::INVALID),
            multicastReplyLocatorList: vec!(LocatorList::INVALID),
            haveTimestamp: false,
            timestamp: TimeStamp::TIME_INVALID,
        }
    }

    pub fn handle_packet(packets: Vec<UdpMessage>) {
        for packet in packets {
            // Is DDSPING
            let msg = packet.message;
            if msg.len() < 20 {
                if msg.len() >= 16 && msg[0..4] == b"RTPS"[..] && msg[9..16] == b"DDSPING"[..] {
                    println!("Received DDSPING");
                    return;
                }
            }
            let packet_buf =msg.freeze();
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
                    Err(e) => panic!("{:?}", e),
                };
                let submessage_body_buf = rtps_body_buf.split_to(submessage_header.get_len() as usize);
                let submessage = SubMessage::new(submessage_header, submessage_body_buf);
                match submessage {
                    Some(msg) => submessages.push(msg),
                    None => println!("received UNKNOWN_RTPS or VENDORSPECIFIC"),
                }
            }
            let rtps_message = Message::new( rtps_header, submessages );
            rtps_message.handle_submessages();
        }
    }
}
