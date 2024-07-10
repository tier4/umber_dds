use crate::discovery::structure::{cdr::deserialize, data::SDPBuiltinData};
use crate::message::{
    submessage::{element::*, submessage_flag::*, *},
    *,
};
use crate::net_util::*;
use crate::rtps::cache::HistoryCache;
use crate::rtps::{
    cache::{CacheChange, ChangeKind, InstantHandle},
    reader::Reader,
    writer::Writer,
};
use crate::structure::entity_id::EntityId;
use crate::structure::{guid::*, vendor_id::*};
use colored::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::{error, fmt};

#[derive(Debug, Clone)]
struct MessageError(String);

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MessageError: ('{}')", self.0)
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
                    eprintln!("<{}>: Received DDSPING", "MessageReceiver: Info".green());
                    return;
                }
            }
            let msg_buf = msg.freeze();
            let rtps_message = match Message::new(msg_buf) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!(
                        "<{}>: couldn't deserialize : {}",
                        "MessageReceiver: Err".red(),
                        e
                    );
                    return;
                }
            };
            eprintln!(
                "<{}>: receive RTPS message form {:?}\n\t{}",
                "MessageReceiver: Info".green(),
                rtps_message.header.guid_prefix,
                rtps_message.summary()
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
        for submsg in rtps_msg.submessages {
            match submsg.body {
                SubMessageBody::Entity(e) => {
                    if let Err(e) = self.handle_entity_submessage(e, &mut writers, &mut readers) {
                        eprintln!("<{}>: {:?}", "MessageReceiver: Err".red(), e);
                        return;
                    }
                }
                SubMessageBody::Interpreter(i) => {
                    if let Err(e) = self.handle_interpreter_submessage(i) {
                        eprintln!("<{}>: {:?}", "MessageReceiver: Err".red(), e);
                        return;
                    }
                }
            }
        }
    }

    fn handle_entity_submessage(
        &mut self,
        entity_subm: EntitySubmessage,
        mut writers: &mut HashMap<EntityId, Writer>,
        mut readers: &mut HashMap<EntityId, Reader>,
    ) -> Result<(), MessageError> {
        match entity_subm {
            EntitySubmessage::AckNack(acknack, flags) => {
                self.handle_acknack_submsg(acknack, flags, &mut writers)
            }
            EntitySubmessage::Data(data, flags) => {
                self.handle_data_submsg(data, flags, &mut readers, &mut writers)
            }
            EntitySubmessage::DataFrag(data_frag, flags) => {
                Self::handle_datafrag_submsg(data_frag, flags)
            }
            EntitySubmessage::Gap(gap, flags) => self.handle_gap_submsg(gap, flags, readers),
            EntitySubmessage::HeartBeat(heartbeat, flags) => {
                self.handle_heartbeat_submsg(heartbeat, flags, readers)
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
        match interpreter_subm {
            InterpreterSubmessage::InfoReply(info_reply, flags) => {
                self.unicast_reply_locator_list = info_reply.unicast_locator_list;
                if flags.contains(InfoReplyFlag::Multicast) {
                    if let Some(multi_loc_list) = info_reply.multicast_locator_list {
                        self.multicast_reply_locator_list = multi_loc_list;
                    } else {
                        return Err(MessageError("Invalid InfoReply Submessage".to_string()));
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
                        return Err(MessageError("Invalid InfoReplyIp4 Submessage".to_string()));
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
                        return Err(MessageError("Invalid InfoTimestamp Submessage".to_string()));
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
                self.dest_guid_prefix = info_dst.guid_prefix;
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
        // rtps 2.3 spec 8.3.7. AckNack

        if self.source_guid_prefix == self.own_guid_prefix
            && self.dest_guid_prefix != GuidPrefix::UNKNOW
        {
            return Err(MessageError(
                "message from same Participant & dest_guid_prefix is not UNKNOWN".to_string(),
            ));
        }

        // validation
        if ackanck.reader_sn_state.bitmap_base == SequenceNumber(0)
            && ackanck.reader_sn_state.num_bits == 0
        {
            // CycloneDDS send bitmap_base = 0, num_bits = 0
            // this is invalid AckNack but, WireShark call this Preemptive ACKNACK
            // I think Preemptive ACKNACK is the ACKNACK message which sent before discover remote
            // reader.
            eprintln!(
                "<{}>: handle Preemptive ACKNACK",
                "MessageReceiver: Info".green(),
            );
            return Ok(());
        }
        if !ackanck.is_valid() {
            return Err(MessageError("Invalid AckNack Submessage".to_string()));
        }

        let _writer_guid = GUID::new(self.dest_guid_prefix, ackanck.writer_id);
        let reader_guid = GUID::new(self.source_guid_prefix, ackanck.reader_id);
        match writers.get_mut(&ackanck.writer_id) {
            Some(w) => w.handle_acknack(ackanck, reader_guid),
            None => (),
        };
        Ok(())
    }
    fn handle_data_submsg(
        &mut self,
        data: Data,
        flag: BitFlags<DataFlag>,
        readers: &mut HashMap<EntityId, Reader>,
        writers: &mut HashMap<EntityId, Writer>,
    ) -> Result<(), MessageError> {
        // rtps 2.3 spec 8.3.7.2 Data

        if self.source_guid_prefix == self.own_guid_prefix
            && self.dest_guid_prefix != GuidPrefix::UNKNOW
        {
            return Err(MessageError(
                "message from same Participant & dest_guid_prefix is not UNKNOWN".to_string(),
            ));
        }

        // validation
        if data.writer_sn < SequenceNumber(0)
            || data.writer_sn == SequenceNumber::SEQUENCENUMBER_UNKNOWN
        {
            return Err(MessageError("Invalid Data Submessage".to_string()));
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
                    return Err(MessageError(format!(
                        "failed deserialize reseived spdp data message: {:?}",
                        e
                    )));
                }
            };
            let new_data = deserialized.to_spdp_discoverd_participant_data();
            let guid_prefix = new_data.guid.guid_prefix;
            eprintln!(
                "<{}>: handle SPDP message from: {:?}",
                "MessageReceiver: Info".green(),
                guid_prefix
            );
            match readers.get_mut(&EntityId::SPDP_BUILTIN_PARTICIPANT_DETECTOR) {
                Some(r) => {
                    eprintln!(
                        "<{}>: add_change to spdp_builtin_participant_reader",
                        "MessageReceiver: Info".green(),
                    );
                    r.matched_writer_add(
                        GUID::new(guid_prefix, EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER),
                        new_data.metarraffic_unicast_locator_list,
                        new_data.metarraffic_multicast_locator_list,
                        0,
                    );
                    r.add_change(self.source_guid_prefix, change)
                }
                None => {
                    eprintln!(
                        "<{}>: couldn't find spdp_builtin_participant_reader",
                        "MessageReceiver: Err".red(),
                    );
                }
            };
        } else if data.writer_id == EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
            || data.reader_id == EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR
        {
            // if msg is for SEDP
            let mut deserialized = match deserialize::<SDPBuiltinData>(
                &data.serialized_payload.as_ref().unwrap().to_bytes(),
            ) {
                Ok(d) => d,
                Err(e) => {
                    return Err(MessageError(format!(
                        "failed deserialize reseived sedp(w) data message: {:?}",
                        e
                    )));
                }
            };
            eprintln!(
                "<{}>: successed for deserialize sedp(w)",
                "MessageReceiver: Info".green()
            );
            let writer_proxy =
                match deserialized.to_writerproxy(Arc::new(RwLock::new(HistoryCache::new()))) {
                    Some(wp) => wp,
                    None => panic!(""),
                };
            let (topic_name, data_type) = match deserialized.topic_info() {
                Some((tn, dt)) => (tn, dt),
                None => panic!(""),
            };
            for (_eid, reader) in readers.iter_mut() {
                if reader.is_writer_match(&topic_name, &data_type) {
                    eprintln!(
                        "<{}>: matched writer add to reader",
                        "MessageReceiver: Info".green()
                    );
                    reader.matched_writer_add(
                        writer_proxy.remote_writer_guid,
                        writer_proxy.unicast_locator_list.clone(),
                        writer_proxy.multicast_locator_list.clone(),
                        writer_proxy.data_max_size_serialized,
                    )
                }
            }
            match readers.get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR) {
                Some(r) => r.add_change(self.source_guid_prefix, change),
                None => {
                    eprintln!(
                        "<{}>: couldn't find sedp_builtin_publication_reader",
                        "MessageReceiver: Err".red(),
                    );
                }
            };
        } else if data.writer_id == EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
            || data.reader_id == EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
        {
            // if msg is for SEDP
            let mut deserialized = match deserialize::<SDPBuiltinData>(
                &data.serialized_payload.as_ref().unwrap().to_bytes(),
            ) {
                Ok(d) => d,
                Err(e) => {
                    return Err(MessageError(format!(
                        "failed deserialize reseived sedp(r) data message: {:?}",
                        e
                    )));
                }
            };
            eprintln!(
                "<{}>: successed for deserialize sedp(r)",
                "MessageReceiver: Info".green()
            );
            let reader_proxy =
                match deserialized.to_readerpoxy(Arc::new(RwLock::new(HistoryCache::new()))) {
                    Some(rp) => rp,
                    None => panic!(""),
                };
            let (topic_name, data_type) = match deserialized.topic_info() {
                Some((tn, dt)) => (tn, dt),
                None => panic!(""),
            };
            for (_eid, writer) in writers.iter_mut() {
                if writer.is_reader_match(&topic_name, &data_type) {
                    eprintln!(
                        "<{}>: matched reader add to writer",
                        "MessageReceiver: Info".green()
                    );
                    writer.matched_reader_add(
                        reader_proxy.remote_reader_guid,
                        reader_proxy.expects_inline_qos,
                        reader_proxy.unicast_locator_list.clone(),
                        reader_proxy.multicast_locator_list.clone(),
                    )
                }
            }
            match readers.get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR) {
                Some(r) => r.add_change(self.source_guid_prefix, change),
                None => {
                    eprintln!(
                        "<{}>: couldn't find sedp_builtin_subscription_reader",
                        "MessageReceiver: Err".red(),
                    );
                }
            };
        } else {
            if data.reader_id == EntityId::UNKNOW {
                for (_eid, reader) in readers {
                    if reader.is_contain_writer(data.writer_id) {
                        reader.add_change(self.source_guid_prefix, change.clone())
                    }
                }
            } else {
                match readers.get_mut(&data.reader_id) {
                    Some(r) => r.add_change(self.source_guid_prefix, change),
                    None => {
                        let mut reader_eids = String::new();
                        for (eid, _reader) in readers.into_iter() {
                            reader_eids += &format!("{:?}\n", eid);
                        }
                        eprintln!(
                            "<{}>: couldn't find reader which has {:?}\nthere are readers which has eid:\n{:?}",
                            "MessageReceiver: Warn".yellow(),
                            data.reader_id,
                            reader_eids
                        );
                    }
                };
            }
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

        if self.source_guid_prefix == self.own_guid_prefix
            && self.dest_guid_prefix != GuidPrefix::UNKNOW
        {
            return Err(MessageError(
                "message from same Participant & dest_guid_prefix is not UNKNOWN".to_string(),
            ));
        }

        // validation
        if !gap.is_valid(flag) {
            return Err(MessageError("Invalid Gap Submessage".to_string()));
        }

        let writer_guid = GUID::new(self.source_guid_prefix, gap.writer_id);
        let _reader_guid = GUID::new(self.dest_guid_prefix, gap.reader_id);
        if gap.reader_id == EntityId::UNKNOW {
            for (_eid, reader) in readers {
                if reader.is_contain_writer(gap.writer_id) {
                    reader.handle_gap(writer_guid, gap.clone());
                }
            }
        } else {
            match readers.get_mut(&gap.reader_id) {
                Some(r) => r.handle_gap(writer_guid, gap),
                None => {
                    let mut reader_eids = String::new();
                    for (eid, _reader) in readers.into_iter() {
                        reader_eids += &format!("{:?}\n", eid);
                    }
                    eprintln!(
                    "<{}>: couldn't find reader which has {:?}\nthere are readers which has eid:\n{}",
                    "MessageReceiver: Warn".yellow(),
                    gap.reader_id,
                    reader_eids
                );
                }
            };
        }
        Ok(())
    }
    fn handle_heartbeat_submsg(
        &self,
        heartbeat: Heartbeat,
        flag: BitFlags<HeartbeatFlag>,
        readers: &mut HashMap<EntityId, Reader>,
    ) -> Result<(), MessageError> {
        // rtps 2.3 spec 8.3.7.5 Heartbeat

        if self.source_guid_prefix == self.own_guid_prefix
            && self.dest_guid_prefix != GuidPrefix::UNKNOW
        {
            return Err(MessageError(
                "message from same Participant & dest_guid_prefix is not UNKNOWN".to_string(),
            ));
        }

        // validation
        if !heartbeat.is_valid(flag) {
            return Err(MessageError("Invalid Heartbeat Submessage".to_string()));
        }

        let writer_guid = GUID::new(self.source_guid_prefix, heartbeat.writer_id);
        let _reader_guid = GUID::new(self.dest_guid_prefix, heartbeat.reader_id);
        let mut reader_eids = String::new();
        for (eid, _reader) in readers.into_iter() {
            reader_eids += &format!("{:?}\n", eid);
        }
        if heartbeat.reader_id == EntityId::UNKNOW {
            match heartbeat.writer_id {
                EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER => {
                    match readers.get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR) {
                        Some(r) => r.handle_heartbeat(writer_guid, flag, heartbeat),
                        None => {
                            unreachable!("not found SEDP_BUILTIN_PUBLICATIONS_DETECTOR");
                        }
                    };
                }
                EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER => {
                    match readers.get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR) {
                        Some(r) => r.handle_heartbeat(writer_guid, flag, heartbeat),
                        None => {
                            unreachable!("not found SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR");
                        }
                    };
                }
                writer_eid => {
                    for (_eid, reader) in readers {
                        if reader.is_contain_writer(writer_eid) {
                            reader.handle_heartbeat(writer_guid, flag, heartbeat.clone());
                        }
                    }
                }
            }
        } else {
            match readers.get_mut(&heartbeat.reader_id) {
                Some(r) => r.handle_heartbeat(writer_guid, flag, heartbeat),
                None => {
                    eprintln!(
                    "<{}>: couldn't find reader which has {:?}\nthere are readers which has eid:\n{}",
                    "MessageReceiver: Warn".yellow(),
                    heartbeat.reader_id,
                    reader_eids
                );
                }
            };
        }
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
