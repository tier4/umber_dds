use crate::dds::qos::DataWriterQosBuilder;
use crate::discovery::discovery_db::DiscoveryDB;
use crate::discovery::structure::{
    cdr::deserialize,
    data::{ParticipantMessageData, ParticipantMessageKind, SDPBuiltinData},
};
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
use crate::structure::{EntityId, GuidPrefix, VendorId, GUID};
use alloc::collections::BTreeMap;
use alloc::fmt;
use alloc::sync::Arc;
use awkernel_sync::rwlock::RwLock;
use log::{debug, error, info, trace, warn};
use mio_extras::channel as mio_channel;
use std::error;

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
    wlp_timer_sender: mio_channel::Sender<EntityId>,
    have_timestamp: bool,
    timestamp: Timestamp,
}

impl MessageReceiver {
    pub fn new(
        participant_guidprefix: GuidPrefix,
        wlp_timer_sender: mio_channel::Sender<EntityId>,
    ) -> MessageReceiver {
        Self {
            own_guid_prefix: participant_guidprefix,
            source_version: ProtocolVersion::PROTOCOLVERSION,
            source_vendor_id: VendorId::VENDORID_UNKNOW,
            source_guid_prefix: GuidPrefix::UNKNOW,
            dest_guid_prefix: GuidPrefix::UNKNOW,
            unicast_reply_locator_list: vec![Locator::INVALID],
            multicast_reply_locator_list: vec![Locator::INVALID],
            wlp_timer_sender,
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
        writers: &mut BTreeMap<EntityId, Writer>,
        readers: &mut BTreeMap<EntityId, Reader>,
        disc_db: &mut DiscoveryDB,
    ) {
        for message in messages {
            // Is DDSPING
            let msg = message.message;
            if msg.len() < 20
                && msg.len() >= 16
                && msg[0..4] == b"RTPS"[..]
                && msg[9..16] == b"DDSPING"[..]
            {
                info!("Received DDSPING");
                return;
            }
            let msg_buf = msg.freeze();
            let rtps_message = match Message::new(msg_buf) {
                Ok(m) => m,
                Err(e) => {
                    error!("couldn't deserialize RTPS message: {}", e);
                    return;
                }
            };
            info!(
                "receive RTPS message form {:?} with {}",
                rtps_message.header.guid_prefix,
                rtps_message.summary()
            );
            self.handle_parsed_packet(rtps_message, writers, readers, disc_db);
        }
    }

    fn handle_parsed_packet(
        &mut self,
        rtps_msg: Message,
        writers: &mut BTreeMap<EntityId, Writer>,
        readers: &mut BTreeMap<EntityId, Reader>,
        disc_db: &mut DiscoveryDB,
    ) {
        self.reset();
        self.dest_guid_prefix = self.own_guid_prefix;
        self.source_guid_prefix = rtps_msg.header.guid_prefix;
        for submsg in rtps_msg.submessages {
            match submsg.body {
                SubMessageBody::Entity(e) => {
                    if let Err(e) = self.handle_entity_submessage(e, writers, readers, disc_db) {
                        error!("{:?}", e); // TODO
                        return;
                    }
                }
                SubMessageBody::Interpreter(i) => {
                    if let Err(e) = self.handle_interpreter_submessage(i) {
                        error!("{:?}", e); // TODO
                        return;
                    }
                }
            }
        }
    }

    fn handle_entity_submessage(
        &mut self,
        entity_subm: EntitySubmessage,
        writers: &mut BTreeMap<EntityId, Writer>,
        readers: &mut BTreeMap<EntityId, Reader>,
        disc_db: &mut DiscoveryDB,
    ) -> Result<(), MessageError> {
        match entity_subm {
            EntitySubmessage::AckNack(acknack, flags) => {
                self.handle_acknack_submsg(acknack, flags, writers)
            }
            EntitySubmessage::Data(data, flags) => {
                self.handle_data_submsg(data, flags, readers, writers, disc_db)
            }
            EntitySubmessage::DataFrag(data_frag, flags) => {
                Self::handle_datafrag_submsg(data_frag, flags)
            }
            EntitySubmessage::Gap(gap, flags) => self.handle_gap_submsg(gap, flags, readers),
            EntitySubmessage::HeartBeat(heartbeat, flags) => {
                self.handle_heartbeat_submsg(heartbeat, flags, readers, disc_db)
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
        _flag: BitFlags<AckNackFlag>,
        writers: &mut BTreeMap<EntityId, Writer>,
    ) -> Result<(), MessageError> {
        // rtps 2.3 spec 8.3.7. AckNack

        if self.source_guid_prefix == self.own_guid_prefix
            && self.dest_guid_prefix != GuidPrefix::UNKNOW
        {
            debug!("message from same Participant & dest_guid_prefix is not UNKNOWN");
            return Ok(());
        }

        let _writer_guid = GUID::new(self.dest_guid_prefix, ackanck.writer_id);
        let reader_guid = GUID::new(self.source_guid_prefix, ackanck.reader_id);

        // validation
        if ackanck.reader_sn_state.bitmap_base == SequenceNumber(0)
            && ackanck.reader_sn_state.num_bits == 0
        {
            // CycloneDDS send bitmap_base = 0, num_bits = 0
            // this is invalid AckNack but, WireShark call this Preemptive ACKNACK
            // I think Preemptive ACKNACK is the ACKNACK message which sent before discover remote
            // reader.
            info!("received Preemptive ACKNACK from Reader {:?}", reader_guid);
            return Ok(());
        }

        if !ackanck.is_valid() {
            return Err(MessageError(format!(
                "Invalid AckNack Submessage from Reader {:?}",
                reader_guid
            )));
        }

        if let Some(w) = writers.get_mut(&ackanck.writer_id) {
            w.handle_acknack(ackanck, reader_guid)
        };
        Ok(())
    }
    fn handle_data_submsg(
        &mut self,
        data: Data,
        flag: BitFlags<DataFlag>,
        readers: &mut BTreeMap<EntityId, Reader>,
        writers: &mut BTreeMap<EntityId, Writer>,
        disc_db: &mut DiscoveryDB,
    ) -> Result<(), MessageError> {
        // rtps 2.3 spec 8.3.7.2 Data

        if self.source_guid_prefix == self.own_guid_prefix
            && self.dest_guid_prefix != GuidPrefix::UNKNOW
        {
            debug!("message from same Participant & dest_guid_prefix is not UNKNOWN");
            return Ok(());
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

        // rtps 2.3 spec, 8.3.7.2.5 Logical Interpretation
        let writer_guid = GUID::new(self.source_guid_prefix, data.writer_id);
        let _reader_guid = GUID::new(self.dest_guid_prefix, data.reader_id);

        let ts = Timestamp::now().expect("failed get Timestamp::new()");
        let change = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            data.writer_sn,
            ts,
            data.serialized_payload.clone(),
            InstantHandle {}, // TODO
        );

        if data.writer_id == EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER
            || data.reader_id == EntityId::SPDP_BUILTIN_PARTICIPANT_DETECTOR
        {
            // if msg is for SPDP
            let mut deserialized =
                match deserialize::<SDPBuiltinData>(&match data.serialized_payload.as_ref() {
                    Some(sp) => sp.to_bytes(),
                    None => {
                        return Err(MessageError(
                            "received spdp message without serializedPayload".to_string(),
                        ))
                    }
                }) {
                    Ok(d) => d,
                    Err(e) => {
                        return Err(MessageError(format!(
                            "failed deserialize reseived spdp data message: {:?}",
                            e
                        )));
                    }
                };
            let new_data = match deserialized.gen_spdp_discoverd_participant_data() {
                Some(nd) => nd,
                None => return Err(MessageError("received incomplete spdp message".to_string())),
            };
            let guid_prefix = new_data.guid.guid_prefix;
            info!("handle SPDP message from: {:?}", guid_prefix);
            match readers.get_mut(&EntityId::SPDP_BUILTIN_PARTICIPANT_DETECTOR) {
                Some(r) => {
                    r.matched_writer_add(
                        GUID::new(guid_prefix, EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER),
                        new_data.metarraffic_unicast_locator_list,
                        new_data.metarraffic_multicast_locator_list,
                        0,
                        DataWriterQosBuilder::new().build(),
                    );
                    r.add_change(self.source_guid_prefix, change)
                }
                None => {
                    error!("not found spdp_builtin_participant_reader");
                }
            };
        } else if data.writer_id == EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
            || data.reader_id == EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR
        {
            // if msg is for SEDP(w)
            let mut deserialized =
                match deserialize::<SDPBuiltinData>(&match data.serialized_payload.as_ref() {
                    Some(sp) => sp.to_bytes(),
                    None => {
                        return Err(MessageError(
                            "received sedp message without serializedPayload".to_string(),
                        ))
                    }
                }) {
                    Ok(d) => d,
                    Err(e) => {
                        return Err(MessageError(format!(
                            "failed deserialize reseived sedp(w) data message: {:?}",
                            e
                        )));
                    }
                };
            let writer_proxy = if let Some(participant_data) =
                disc_db.read_participant_data(self.source_guid_prefix)
            {
                let default_unicast_locator_list = participant_data.default_unicast_locator_list;
                let default_multicast_locator_list =
                    participant_data.default_multicast_locator_list;
                match deserialized.gen_writerproxy(
                    Arc::new(RwLock::new(HistoryCache::new())),
                    default_unicast_locator_list,
                    default_multicast_locator_list,
                ) {
                    Some(wp) => wp,
                    None => {
                        return Err(MessageError(
                            "failed generate writer_proxy form received DATA(w)".to_string(),
                        ));
                    }
                }
            } else {
                return Err(MessageError(
                    "received sedp(w) from unknown participant".to_string(),
                ));
            };
            let (topic_name, data_type) = match deserialized.topic_info() {
                Some((tn, dt)) => (tn, dt),
                None => {
                    return Err(MessageError(
                        "falied to get topic_info from received DATA(w)".to_string(),
                    ));
                }
            };
            for (eid, reader) in readers.iter_mut() {
                if reader.is_writer_match(&topic_name, &data_type) {
                    let remote_writer_topic_kind =
                        writer_proxy.remote_writer_guid.entity_id.topic_kind();
                    if let Some(tk) = remote_writer_topic_kind {
                        if reader.topic_kind() != tk {
                            error!(
                                "Reader found Writer which has same topic_name: {} & data_type: {} , but not match topic_kind\n\tReader: {:?}\n\tWriter: {:?}",
                                topic_name, data_type,
                                reader.topic_kind(),
                                tk,
                            );
                            return Ok(());
                        }
                    }
                    info!(
                        "Reader found matched Writer Reader: {:?}\n\tWriter: {:?}",
                        eid, writer_proxy.remote_writer_guid
                    );
                    reader.matched_writer_add_with_default_locator(
                        writer_proxy.remote_writer_guid,
                        writer_proxy.unicast_locator_list.clone(),
                        writer_proxy.multicast_locator_list.clone(),
                        writer_proxy.default_unicast_locator_list.clone(),
                        writer_proxy.default_multicast_locator_list.clone(),
                        writer_proxy.data_max_size_serialized,
                        writer_proxy.qos.clone(),
                    );
                    self.wlp_timer_sender
                        .send(*eid)
                        .expect("couldn't send channel 'wlp_timer_sender'");
                }
            }
            match readers.get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR) {
                Some(r) => r.add_change(self.source_guid_prefix, change),
                None => {
                    return Err(MessageError(
                        "not find sedp_builtin_publication_reader".to_string(),
                    ));
                }
            };
        } else if data.writer_id == EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
            || data.reader_id == EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
        {
            // if msg is for SEDP(r)
            let mut deserialized =
                match deserialize::<SDPBuiltinData>(&match data.serialized_payload.as_ref() {
                    Some(sp) => sp.to_bytes(),
                    None => {
                        return Err(MessageError(
                            "received sedp message without serializedPayload".to_string(),
                        ))
                    }
                }) {
                    Ok(d) => d,
                    Err(e) => {
                        return Err(MessageError(format!(
                            "failed deserialize reseived sedp(r) data message: {:?}",
                            e
                        )));
                    }
                };
            let reader_proxy = if let Some(participant_data) =
                disc_db.read_participant_data(self.source_guid_prefix)
            {
                let default_unicast_locator_list = participant_data.default_unicast_locator_list;
                let default_multicast_locator_list =
                    participant_data.default_multicast_locator_list;
                match deserialized.gen_readerpoxy(
                    Arc::new(RwLock::new(HistoryCache::new())),
                    default_unicast_locator_list,
                    default_multicast_locator_list,
                ) {
                    Some(rp) => rp,
                    None => {
                        return Err(MessageError(
                            "failed generate reader_proxy form received DATA(r)".to_string(),
                        ));
                    }
                }
            } else {
                return Err(MessageError(
                    "received sedp(r) from unknown participant".to_string(),
                ));
            };
            let (topic_name, data_type) = match deserialized.topic_info() {
                Some((tn, dt)) => (tn, dt),
                None => {
                    return Err(MessageError(
                        "falied to get topic_info from received DATA(p)".to_string(),
                    ));
                }
            };
            for (eid, writer) in writers.iter_mut() {
                if writer.is_reader_match(&topic_name, &data_type) {
                    let remote_reader_topic_kind =
                        reader_proxy.remote_reader_guid.entity_id.topic_kind();
                    if let Some(tk) = remote_reader_topic_kind {
                        if writer.topic_kind() != tk {
                            error!(
                                "Writer found Reader which has same topic_name: {} & data_type: {} , but not match topic_kind\n\tReader: {:?}\n\tWriter: {:?}",
                                topic_name, data_type,
                                writer.topic_kind(),
                                tk,
                            );
                            return Ok(());
                        }
                    }
                    info!(
                        "Writer found matched Reader\n\tWriter: {:?}\n\tWriter: {:?}",
                        eid, reader_proxy.remote_reader_guid
                    );
                    writer.matched_reader_add_with_default_locator(
                        reader_proxy.remote_reader_guid,
                        reader_proxy.expects_inline_qos,
                        reader_proxy.unicast_locator_list.clone(),
                        reader_proxy.multicast_locator_list.clone(),
                        reader_proxy.default_unicast_locator_list.clone(),
                        reader_proxy.default_multicast_locator_list.clone(),
                        reader_proxy.qos.clone(),
                    )
                }
            }
            match readers.get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR) {
                Some(r) => r.add_change(self.source_guid_prefix, change),
                None => {
                    return Err(MessageError(
                        "not find sedp_builtin_subscription_reader".to_string(),
                    ));
                }
            };
        } else if data.writer_id == EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER
            || data.reader_id == EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER
        {
            // if ParticipantMessage
            let deserialized =
                match deserialize::<ParticipantMessageData>(&match data.serialized_payload.as_ref()
                {
                    Some(sp) => sp.to_bytes(),
                    None => {
                        return Err(MessageError(
                            "received participant message without serializedPayload".to_string(),
                        ))
                    }
                }) {
                    Ok(d) => d,
                    Err(e) => {
                        return Err(MessageError(format!(
                            "failed deserialize reseived sedp(r) data message: {:?}",
                            e
                        )));
                    }
                };
            match deserialized.kind {
                ParticipantMessageKind::MANUAL_LIVELINESS_UPDATE
                | ParticipantMessageKind::AUTOMATIC_LIVELINESS_UPDATE => {
                    info!(
                        "receved DATA(m) with ParticipantMessageKind::{{MANUAL_LIVELINESS_UPDATE or AUTOMATIC_LIVELINESS_UPDATE}} from Participant: {:?}",
                         self.source_guid_prefix
                    );
                    disc_db.write_remote_writer(deserialized.guid, ts);
                }
                ParticipantMessageKind::UNKNOWN => {
                    warn!(
                        "receved DATA(m) with ParticipantMessageKind::UNKNOWN, which is not processed",

                    );
                }
                k => {
                    warn!(
                        "receved DATA(m) with ParticipantMessageKind::{:?}, which is not processed",
                        k.value
                    );
                }
            }
            /*
            match readers.get_mut(&EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER) {
                Some(r) => r.add_change(self.source_guid_prefix, change),
                None => {
                    return Err(MessageError(
                        "not find p2p_builtin_participant_reader".to_string(),
                    ));
                }
            };
            */
        } else if data.reader_id == EntityId::UNKNOW {
            for reader in readers.values_mut() {
                if reader.is_contain_writer(GUID::new(self.source_guid_prefix, data.writer_id)) {
                    disc_db.write_remote_writer(writer_guid, ts);
                    reader.add_change(self.source_guid_prefix, change.clone())
                }
            }
        } else {
            match readers.get_mut(&data.reader_id) {
                Some(r) => {
                    disc_db.write_remote_writer(writer_guid, ts);
                    r.add_change(self.source_guid_prefix, change)
                }
                None => {
                    warn!(
                        "recieved DATA message to unknown Reader {:?}",
                        data.reader_id
                    );
                }
            };
        }
        Ok(())
    }
    fn handle_datafrag_submsg(
        _data_frag: DataFrag,
        _flag: BitFlags<DataFragFlag>,
    ) -> Result<(), MessageError> {
        Ok(())
    }
    fn handle_gap_submsg(
        &self,
        gap: Gap,
        flag: BitFlags<GapFlag>,
        readers: &mut BTreeMap<EntityId, Reader>,
    ) -> Result<(), MessageError> {
        // rtps 2.3 spec 8.3.7.4 Gap

        if self.source_guid_prefix == self.own_guid_prefix
            && self.dest_guid_prefix != GuidPrefix::UNKNOW
        {
            debug!("message from same Participant & dest_guid_prefix is not UNKNOWN");
            return Ok(());
        }

        let writer_guid = GUID::new(self.source_guid_prefix, gap.writer_id);
        let _reader_guid = GUID::new(self.dest_guid_prefix, gap.reader_id);

        // validation
        if !gap.is_valid(flag) {
            return Err(MessageError(format!(
                "Invalid Gap Submessage from Writer {:?}",
                writer_guid,
            )));
        }

        if gap.reader_id == EntityId::UNKNOW {
            for reader in readers.values_mut() {
                if reader.is_contain_writer(GUID::new(self.source_guid_prefix, gap.writer_id)) {
                    reader.handle_gap(writer_guid, gap.clone());
                }
            }
        } else {
            match readers.get_mut(&gap.reader_id) {
                Some(r) => r.handle_gap(writer_guid, gap),
                None => {
                    warn!("receved GAP message to unknown Reader {:?}", gap.reader_id);
                }
            };
        }
        Ok(())
    }
    fn handle_heartbeat_submsg(
        &self,
        heartbeat: Heartbeat,
        flag: BitFlags<HeartbeatFlag>,
        readers: &mut BTreeMap<EntityId, Reader>,
        disc_db: &mut DiscoveryDB,
    ) -> Result<(), MessageError> {
        // rtps 2.3 spec 8.3.7.5 Heartbeat

        if self.source_guid_prefix == self.own_guid_prefix
            && self.dest_guid_prefix != GuidPrefix::UNKNOW
        {
            debug!("message from same Participant & dest_guid_prefix is not UNKNOWN");
            return Ok(());
        }

        let writer_guid = GUID::new(self.source_guid_prefix, heartbeat.writer_id);
        let _reader_guid = GUID::new(self.dest_guid_prefix, heartbeat.reader_id);

        // validation
        if !heartbeat.is_valid(flag) {
            return Err(MessageError(format!(
                "Invalid Heartbeat Submessage from Writer {:?}",
                writer_guid
            )));
        }

        let ts = Timestamp::now().expect("failed get Timestamp::new()");

        let mut reader_eids = String::new();
        for (eid, _reader) in readers.iter_mut() {
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
                    for reader in readers.values_mut() {
                        if reader.is_contain_writer(GUID::new(self.source_guid_prefix, writer_eid))
                        {
                            disc_db.write_remote_writer(writer_guid, ts);
                            reader.handle_heartbeat(writer_guid, flag, heartbeat.clone());
                        }
                    }
                }
            }
        } else {
            match readers.get_mut(&heartbeat.reader_id) {
                Some(r) => {
                    disc_db.write_remote_writer(writer_guid, ts);
                    r.handle_heartbeat(writer_guid, flag, heartbeat)
                }
                None => {
                    warn!(
                        "receved HEARTBEAT message to unknown Reader {:?}",
                        heartbeat.reader_id,
                    );
                }
            };
        }
        Ok(())
    }
    fn handle_heartbeatfrag_submsg(
        _heartbeat_frag: HeartbeatFrag,
        _flag: BitFlags<HeartbeatFragFlag>,
    ) -> Result<(), MessageError> {
        Ok(())
    }
    fn handle_nackfrag_submsg(
        _nack_frag: NackFrag,
        _flag: BitFlags<NackFragFlag>,
    ) -> Result<(), MessageError> {
        Ok(())
    }
}
