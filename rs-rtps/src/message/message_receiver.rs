use crate::discovery::structure::{cdr::deserialize, data::SDPBuiltinData};
use crate::message::{
    submessage::{element::*, submessage_flag::*, *},
    *,
};
use crate::net_util::*;
use crate::rtps::{
    cache::{CacheChange, ChangeKind, InstantHandle},
    reader::Reader,
    writer::Writer,
};
use crate::structure::entity_id::EntityId;
use crate::structure::{guid::*, vendor_id::*};
use colored::*;
use std::collections::HashMap;
use std::{error, fmt};

#[derive(Debug, Clone)]
struct MessageError;

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

impl error::Error for MessageError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

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

    pub fn handle_packet(
        &mut self,
        messages: Vec<UdpMessage>,
        mut writers: &mut HashMap<EntityId, Writer>,
        mut readers: &mut HashMap<EntityId, Reader>,
    ) {
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
            if rtps_message.header.guid_prefix == self.own_guid_prefix {
                eprintln!("*****  RTPS message form self. *****");
                return;
            }
            eprintln!("self.own_guid_prefix: {:?}", self.own_guid_prefix);
            eprintln!(
                "*****  RTPS message form {:?}. *****",
                rtps_message.header.guid_prefix
            );
            self.handle_parsed_packet(rtps_message, &mut writers, &mut readers);
        }
    }

    fn handle_parsed_packet(
        &mut self,
        rtps_msg: Message,
        mut writers: &mut HashMap<EntityId, Writer>,
        mut readers: &mut HashMap<EntityId, Reader>,
    ) {
        self.reset();
        self.dest_guid_prefix = self.own_guid_prefix;
        self.source_guid_prefix = rtps_msg.header.guid_prefix;
        println!(">>>>>>>>>>>>>>>>");
        println!("header: {:?}", rtps_msg.header);
        for submsg in rtps_msg.submessages {
            println!("submessage header: {:?}", submsg.header);
            println!("submessage kind: {:?}", submsg.header.get_submessagekind());
            match submsg.body {
                SubMessageBody::Entity(e) => {
                    if self
                        .handle_entity_submessage(e, &mut writers, &mut readers)
                        .is_err()
                    {
                        return;
                    }
                }
                SubMessageBody::Interpreter(i) => {
                    if self.handle_interpreter_submessage(i).is_err() {
                        return;
                    }
                }
            }
        }
        println!("<<<<<<<<<<<<<<<<\n");
    }

    fn handle_entity_submessage(
        &mut self,
        entity_subm: EntitySubmessage,
        mut writers: &mut HashMap<EntityId, Writer>,
        mut readers: &mut HashMap<EntityId, Reader>,
    ) -> Result<(), MessageError> {
        println!("handle entity submsg");
        match entity_subm {
            EntitySubmessage::AckNack(acknack, flags) => {
                self.handle_acknack_submsg(acknack, flags, &mut writers)
            }
            EntitySubmessage::Data(data, flags) => {
                self.handle_data_submsg(data, flags, &mut readers)
            }
            EntitySubmessage::DataFrag(data_frag, flags) => {
                Self::handle_datafrag_submsg(data_frag, flags)
            }
            EntitySubmessage::Gap(gap, flags) => self.handle_gap_submsg(gap, flags, readers),
            EntitySubmessage::HeartBeat(heartbeat, flags) => {
                Self::handle_heartbeat_submsg(heartbeat, flags)
            }
            EntitySubmessage::HeartbeatFrag(heartbeatfrag, flags) => {
                Self::handle_heartbeatfrag_submsg(heartbeatfrag, flags)
            }
            EntitySubmessage::NackFrag(nack_frag, flags) => {
                Self::handle_nackfrag_submsg(nack_frag, flags)
            }
        }
    }
    fn handle_interpreter_submessage(
        &mut self,
        interpreter_subm: InterpreterSubmessage,
    ) -> Result<(), MessageError> {
        println!("handle interpreter submsg");
        match interpreter_subm {
            InterpreterSubmessage::InfoReply(info_reply, flags) => {
                self.unicast_reply_locator_list = info_reply.unicast_locator_list;
                if flags.contains(InfoReplyFlag::Multicast) {
                    if let Some(multi_loc_list) = info_reply.multicast_locator_list {
                        self.multicast_reply_locator_list = multi_loc_list;
                    } else {
                        return Err(MessageError);
                    }
                } else {
                    self.multicast_reply_locator_list.clear();
                }
            }
            InterpreterSubmessage::InfoReplyIp4(info_reply_ip4, flags) => {
                self.unicast_reply_locator_list = info_reply_ip4.unicast_locator_list;
                if flags.contains(InfoReplyIp4Flag::Multicast) {
                    if let Some(multi_rep_loc_list) = info_reply_ip4.multicast_locator_list {
                        self.multicast_reply_locator_list = multi_rep_loc_list;
                    } else {
                        return Err(MessageError);
                    }
                } else {
                    self.multicast_reply_locator_list.clear();
                }
            }
            InterpreterSubmessage::InfoTimestamp(info_ts, flags) => {
                if !flags.contains(InfoTimestampFlag::Invalidate) {
                    self.have_timestamp = true;
                    if let Some(ts) = info_ts.timestamp {
                        self.timestamp = ts;
                    } else {
                        return Err(MessageError);
                    }
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
            InterpreterSubmessage::InfoDestination(info_dst, _flags) => {
                if info_dst.guid_prefix != GuidPrefix::UNKNOW {
                    self.dest_guid_prefix = info_dst.guid_prefix;
                } else {
                    self.dest_guid_prefix = self.own_guid_prefix;
                }
            }
        }
        Ok(())
    }

    fn handle_acknack_submsg(
        &self,
        ackanck: AckNack,
        flag: BitFlags<AckNackFlag>,
        writers: &mut HashMap<EntityId, Writer>,
    ) -> Result<(), MessageError> {
        // validation
        if !ackanck.reader_sn_state.is_valid() {
            eprintln!("Invalid AckNack Submessage received");
            return Err(MessageError);
        }
        let writer_guid = GUID::new(self.dest_guid_prefix, ackanck.writer_id);
        let reader_guid = GUID::new(self.source_guid_prefix, ackanck.reader_id);
        // TODO: writer.handle_acknackに適切な引数を渡す。
        match writers.get_mut(&ackanck.writer_id) {
            Some(w) => w.handle_acknack(),
            None => (),
        };
        Ok(())
    }
    fn handle_data_submsg(
        &mut self,
        data: Data,
        flag: BitFlags<DataFlag>,
        readers: &mut HashMap<EntityId, Reader>,
    ) -> Result<(), MessageError> {
        // rtps 2.3 spec 8.3.7.2 Data
        // validation
        if data.writer_sn < SequenceNumber(0)
            || data.writer_sn == SequenceNumber::SEQUENCENUMBER_UNKNOWN
        {
            eprintln!("Invalid Data Submessage received");
            return Err(MessageError);
        }
        // TODO: check inlineQos is valid
        if flag.contains(DataFlag::Data) && !flag.contains(DataFlag::Key) {
            // he serializedPayload element is interpreted as the value of the dtat-object
        }
        if flag.contains(DataFlag::Key) && !flag.contains(DataFlag::Data) {
            // the serializedPayload element is interpreted as the value of the key that identifies the registered instance of the data-object.
        }
        if flag.contains(DataFlag::InlineQos) {
            // the inlineQos element contains QoS values that override those of the RTPS Writer and should
            // be used to process the update. For a complete list of possible in-line QoS parameters, see Table 8.80.
        }
        if flag.contains(DataFlag::NonStandardPayload) {
            // the serializedPayload element is not formatted according to Section 10.
            // This flag is informational. It indicates that the SerializedPayload has been transformed as described in another specification
            // For example, this flag should be set when the SerializedPayload is transformed as described in the DDS-Security specification
        }
        let writer_guid = GUID::new(self.dest_guid_prefix, data.writer_id);
        let _reader_guid = GUID::new(self.source_guid_prefix, data.reader_id);

        let change = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            data.writer_sn,
            data.serialized_payload.clone(),
            InstantHandle {}, // TODO
        );

        if data.writer_id == EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER
            || data.reader_id == EntityId::SPDP_BUILTIN_PARTICIPANT_DETECTOR
        {
            // if msg is for SPDP
            let mut deserialized = match deserialize::<SDPBuiltinData>(
                &data.serialized_payload.as_ref().unwrap().to_bytes(),
            ) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!(
                        "neko~~~~~: failed deserialize reseived spdp data message: {}",
                        e
                    );
                    return Err(MessageError);
                }
            };
            let new_data = deserialized.to_spdp_discoverd_participant_data();
            let guid_prefix = new_data.guid.guid_prefix;
            println!(
                "##################  @message_receiver  Discovery message received from: {:?}",
                guid_prefix
            );
            /*
            eprintln!("domain_id: {}", new_data.domain_id);
            eprintln!("domain_tag: {}", new_data.domain_tag);
            eprintln!("protocol_version: {:?}", new_data.protocol_version);
            eprintln!("guid: {:?}", new_data.guid);
            eprintln!("vendor_id: {:?}", new_data.vendor_id);
            eprintln!("expects_inline_qos: {:?}", new_data.expects_inline_qos);
            eprintln!(
                "available_builtin_endpoint: {:?}",
                new_data.available_builtin_endpoint
            );
            eprintln!(
                "metarraffic_unicast_locator_list: {:?}",
                new_data.metarraffic_unicast_locator_list
            );
            eprintln!(
                "metarraffic_multicast_locator_list: {:?}",
                new_data.metarraffic_multicast_locator_list
            );
            eprintln!(
                "default_unicast_locator_list: {:?}",
                new_data.default_unicast_locator_list
            );
            eprintln!(
                "default_multicast_locator_list: {:?}",
                new_data.default_multicast_locator_list
            );
            eprintln!(
                "manual_liveliness_count: {:?}",
                new_data.manual_liveliness_count
            );
            eprintln!("lease_duration: {:?}", new_data.lease_duration.clone());
            */
            match readers.get_mut(&EntityId::SPDP_BUILTIN_PARTICIPANT_DETECTOR) {
                Some(r) => r.add_change(change),
                None => (),
            };
        } else if data.writer_id == EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
            || data.reader_id == EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR
        {
            // if msg is for SEDP
            let deserialized = match deserialize::<SDPBuiltinData>(
                &data.serialized_payload.as_ref().unwrap().to_bytes(),
            ) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!(
                        "neko~~~~~: failed deserialize reseived sedp(w) data message: {}",
                        e
                    );
                    return Err(MessageError);
                }
            };
            eprintln!("successed for deserialize sedp(w)");
            match readers.get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR) {
                Some(r) => r.add_change(change),
                None => (),
            };
        } else if data.writer_id == EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
            || data.reader_id == EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
        {
            // if msg is for SEDP
            let deserialized = match deserialize::<SDPBuiltinData>(
                &data.serialized_payload.as_ref().unwrap().to_bytes(),
            ) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!(
                        "neko~~~~~: failed deserialize reseived sedp(r) data message: {}",
                        e
                    );
                    return Err(MessageError);
                }
            };
            eprintln!("successed for deserialize sedp(r)");
            match readers.get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR) {
                Some(r) => r.add_change(change),
                None => (),
            };
        } else {
            match readers.get_mut(&data.reader_id) {
                Some(r) => r.add_change(change),
                None => match readers.get_mut(&EntityId::SPDP_BUILTIN_PARTICIPANT_DETECTOR) {
                    Some(r) => r.add_change(change),
                    None => (),
                },
            };
        }
        Ok(())
    }
    fn handle_datafrag_submsg(
        data_frag: DataFrag,
        flag: BitFlags<DataFragFlag>,
    ) -> Result<(), MessageError> {
        Ok(())
    }
    fn handle_gap_submsg(
        &self,
        gap: Gap,
        flag: BitFlags<GapFlag>,
        readers: &mut HashMap<EntityId, Reader>,
    ) -> Result<(), MessageError> {
        // rtps 2.3 spec 8.3.7.4 Gap
        // validation
        if gap.gap_start <= SequenceNumber(0) {
            // gapStart is zero or negative
            eprintln!("Invalid Gap Submessage received");
            return Err(MessageError);
        }
        if !gap.gap_list.is_valid() {
            // gapList is invalid
            eprintln!("Invalid Gap Submessage received");
            return Err(MessageError);
        }
        if flag.contains(GapFlag::GroupInfo) {
            // GroupInfoFlag is set and
            if gap.gap_start_gsn <= SequenceNumber(0) {
                // gapStartGSN.value is zero or negative
                eprintln!("Invalid Gap Submessage received");
                return Err(MessageError);
            }
            if gap.gap_end_gsn <= SequenceNumber(0) {
                // gapEndGSN.value is zero or negative
                eprintln!("Invalid Gap Submessage received");
                return Err(MessageError);
            }
            if gap.gap_end_gsn < gap.gap_start_gsn - SequenceNumber(1) {
                // gapEndGSN.value < gapStartGSN.value-1
                eprintln!("Invalid Gap Submessage received");
                return Err(MessageError);
            }
        }
        let _writer_guid = GUID::new(self.source_guid_prefix, gap.writer_id);
        let _reader_guid = GUID::new(self.dest_guid_prefix, gap.reader_id);
        match readers.get_mut(&gap.reader_id) {
            Some(r) => r.handle_gqp(gap),
            None => (),
        };
        Ok(())
    }
    fn handle_heartbeat_submsg(
        heartbeat: Heartbeat,
        flag: BitFlags<HeartbeatFlag>,
    ) -> Result<(), MessageError> {
        Ok(())
    }
    fn handle_heartbeatfrag_submsg(
        heartbeat_frag: HeartbeatFrag,
        flag: BitFlags<HeartbeatFragFlag>,
    ) -> Result<(), MessageError> {
        Ok(())
    }
    fn handle_nackfrag_submsg(
        nack_frag: NackFrag,
        flag: BitFlags<NackFragFlag>,
    ) -> Result<(), MessageError> {
        Ok(())
    }
}
