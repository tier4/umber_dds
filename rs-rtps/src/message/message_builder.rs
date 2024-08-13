use super::element::*;
use super::{
    submessage::{
        element::{
            acknack::AckNack, data::Data, gap::Gap, heartbeat::Heartbeat, infodst::InfoDestination,
            infots::InfoTimestamp,
        },
        submessage_flag::{
            AckNackFlag, DataFlag, GapFlag, HeartbeatFlag, InfoDestionationFlag, InfoTimestampFlag,
        },
        submessage_header::SubMessageHeader,
        EntitySubmessage, InterpreterSubmessage, SubMessage, SubMessageBody, SubMessageKind,
    },
    Header, Message,
};
use crate::rtps::cache::CacheChange;
use crate::structure::{entity_id::EntityId, guid::GuidPrefix};
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

    pub fn info_ts(&mut self, endiannes: Endianness, timestamp: Option<Timestamp>) {
        let mut ts_flag = InfoTimestampFlag::from_enndianness(endiannes);
        let mut infots_body_length = 0;
        let ts = match timestamp {
            Some(t) => t,
            None => {
                ts_flag |= InfoTimestampFlag::Invalidate;
                Timestamp::TIME_INVALID
            }
        };
        infots_body_length += 8;
        let info_ts = InfoTimestamp {
            timestamp: Some(ts),
        };
        let ts_body =
            SubMessageBody::Interpreter(InterpreterSubmessage::InfoTimestamp(info_ts, ts_flag));
        let ts_header = SubMessageHeader::new(
            SubMessageKind::INFO_TS as u8,
            ts_flag.bits(),
            infots_body_length,
        );
        let ts_msg = SubMessage {
            header: ts_header,
            body: ts_body,
        };
        self.submessages.push(ts_msg);
    }

    pub fn info_dst(&mut self, endiannes: Endianness, destination: GuidPrefix) {
        let dst_flag = InfoDestionationFlag::from_enndianness(endiannes);
        let infodst_body_length = 12;
        let info_dst = InfoDestination {
            guid_prefix: destination,
        };
        let dst_body =
            SubMessageBody::Interpreter(InterpreterSubmessage::InfoDestination(info_dst, dst_flag));
        let dst_header = SubMessageHeader::new(
            SubMessageKind::INFO_DST as u8,
            dst_flag.bits(),
            infodst_body_length,
        );
        let dst_msg = SubMessage {
            header: dst_header,
            body: dst_body,
        };
        self.submessages.push(dst_msg);
    }

    pub fn acknack(
        &mut self,
        endiannes: Endianness,
        writer_id: EntityId,
        reader_id: EntityId,
        reader_sn_state: SequenceNumberSet,
        count: Count,
        is_final: bool,
    ) {
        let acknack_body_length = 12 + reader_sn_state.size();
        let acknack = AckNack::new(reader_id, writer_id, reader_sn_state, count);
        let mut acknack_flag = AckNackFlag::from_enndianness(endiannes);
        if is_final {
            acknack_flag |= AckNackFlag::Final;
        }
        let acknack_body = SubMessageBody::Entity(EntitySubmessage::AckNack(acknack, acknack_flag));
        let acknack_header = SubMessageHeader::new(
            SubMessageKind::ACKNACK as u8,
            acknack_flag.bits(),
            acknack_body_length,
        );
        let acknack_msg = SubMessage {
            header: acknack_header,
            body: acknack_body,
        };
        self.submessages.push(acknack_msg);
    }

    pub fn heartbeat(
        &mut self,
        endiannes: Endianness,
        writer_id: EntityId,
        reader_id: EntityId,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
        count: Count,
        is_final: bool,
    ) {
        let heartbeat_body_length = 28;
        let hb = Heartbeat::new(
            reader_id, writer_id, first_sn, last_sn, count, None, None, None, None, None,
        );
        let mut hb_flag = HeartbeatFlag::from_enndianness(endiannes);
        if is_final {
            hb_flag |= HeartbeatFlag::Final;
        }
        let hb_body = SubMessageBody::Entity(EntitySubmessage::HeartBeat(hb, hb_flag));
        let hb_header = SubMessageHeader::new(
            SubMessageKind::HEARTBEAT as u8,
            hb_flag.bits(),
            heartbeat_body_length,
        );
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
    ) {
        let mut data_flag = DataFlag::from_enndianness(endiannes);
        let payload_length;
        let serialized_payload = cache_change.data_value();
        if let Some(payload) = &serialized_payload {
            data_flag |= DataFlag::Data;
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

    pub fn gap(
        &mut self,
        endiannes: Endianness,
        writer_id: EntityId,
        reader_id: EntityId,
        gap_start: SequenceNumber,
        gap_list: SequenceNumberSet,
    ) {
        let gap_body_length = 16 + gap_list.size();
        let gap = Gap::new(reader_id, writer_id, gap_start, gap_list);
        let gap_flag = GapFlag::from_enndianness(endiannes);
        let gap_body = SubMessageBody::Entity(EntitySubmessage::Gap(gap, gap_flag));
        let gap_header =
            SubMessageHeader::new(SubMessageKind::GAP as u8, gap_flag.bits(), gap_body_length);
        let gap_msg = SubMessage {
            header: gap_header,
            body: gap_body,
        };
        self.submessages.push(gap_msg);
    }

    pub fn build(self, self_guid_prefix: GuidPrefix) -> Message {
        Message {
            header: Header::new(self_guid_prefix),
            submessages: self.submessages,
        }
    }
}
