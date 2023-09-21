use super::element::*;
use super::{
    submessage::{
        element::{data::Data, heartbeat::Heartbeat, infots::InfoTimestamp},
        submessage_flag::{DataFlag, HeartbeatFlag, InfoTimestampFlag},
        submessage_header::SubMessageHeader,
        EntitySubmessage, InterpreterSubmessage, SubMessage, SubMessageBody, SubMessageKind,
    },
    Header, Message,
};
use crate::rtps::cache::CacheChange;
use crate::structure::{entity_id::EntityId, guid::GuidPrefix};
use bytes::Bytes;
use enumflags2::{bitflags, BitFlags};
use speedy::Endianness;

pub struct MessageBuilder {
    submessages: Vec<SubMessage>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            submessages: Vec::new(),
        }
    }

    pub fn info_ts(&self, endiannes: Endianness) {
        let info_ts = todo!();
        let ts_flag = InfoTimestampFlag::from_enndianness(endiannes);
        let ts_body =
            SubMessageBody::Interpreter(InterpreterSubmessage::InfoTimestamp(info_ts, ts_flag));
        let ts_header =
            SubMessageHeader::new(SubMessageKind::INFO_TS as u8, ts_flag.bits(), todo!());
        let ts_msg = SubMessage {
            header: ts_header,
            body: ts_body,
        };
        self.submessages.push(ts_msg);
    }

    pub fn heartbeat(&self, endiannes: Endianness) {
        let hb = todo!();
        let hb_flag = HeartbeatFlag::from_enndianness(endiannes);
        let hb_body = SubMessageBody::Entity(EntitySubmessage::HeartBeat(hb, hb_flag));
        let hb_header =
            SubMessageHeader::new(SubMessageKind::HEARTBEAT as u8, hb_flag.bits(), todo!());
        let hb_msg = SubMessage {
            header: hb_header,
            body: hb_body,
        };
        self.submessages.push(hb_msg);
    }

    pub fn data(
        &mut self,
        endiannes: Endianness,
        reader_id: EntityId,
        writer_id: EntityId,
        cache_change: CacheChange,
        serialized_payload: Option<SerializedPayload>,
    ) {
        let mut data_flag = DataFlag::from_enndianness(endiannes);
        let payload_length;
        if let Some(payload) = &serialized_payload {
            data_flag |= DataFlag::Datqa;
            payload_length = 4 + payload.value.len();
        } else {
            payload_length = 0;
        }
        let inline_qos_len = 0;
        let data = Data::new(
            reader_id,
            writer_id,
            cache_change.sequence_number,
            None,
            serialized_payload,
        );
        let data_body = SubMessageBody::Entity(EntitySubmessage::Data(data, data_flag));
        // extra_flags(2), octets_to_inlineQos(2), reader_id(4), writer_id(4), writer_sn(8) octet
        let data_body_length = 2 + 2 + 4 + 4 + 8 + inline_qos_len + payload_length;
        let data_header = SubMessageHeader::new(
            SubMessageKind::DATA as u8,
            data_flag.bits(),
            data_body_length as u16,
        );
        let data_msg = SubMessage {
            header: data_header,
            body: data_body,
        };
        self.submessages.push(data_msg);
    }

    pub fn build(self, guid_prefix: GuidPrefix) -> Message {
        Message {
            header: Header::new(guid_prefix),
            submessages: self.submessages,
        }
    }
}
