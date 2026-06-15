use crate::dds::qos::{
    policy::{Durability, History, HistoryQosKind, LivelinessQosKind, Reliability},
    DataReaderQosBuilder, DataWriterQosBuilder,
};
use crate::discovery::discovery_db::DiscoveryDB;
use crate::discovery::structure::{
    builtin_endpoint::BuiltinEndpoint,
    cdr::deserialize,
    data::{
        ParticipantMessageData, ParticipantMessageKind, SDPBuiltinData,
        SPDPdiscoveredParticipantData,
    },
};
use crate::message::{
    submessage::{element::*, submessage_flag::*, *},
    *,
};
use crate::net_util::*;
use crate::rtps::cache::{HistoryCache, HistoryCacheType};
use crate::rtps::{
    cache::{CacheChange, ChangeKind, InstantHandle},
    reader::{Reader, ReaderTimer},
    writer::{Writer, WriterTimer},
};
use crate::structure::{EntityId, GuidPrefix, VendorId, GUID};
use alloc::collections::BTreeMap;
use alloc::fmt;
use alloc::sync::Arc;
use awkernel_sync::rwlock::RwLock;
use log::{error, info, trace, warn};
use mio_extras::channel as mio_channel;
use speedy::Endianness;
use std::error;

#[derive(Debug, Clone)]
enum MessageError {
    Error(String),
    Warn(String),
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Error(e) => write!(f, "MessageError: ('{}')", e),
            Self::Warn(w) => write!(f, "MessageWarn: ('{}')", w),
        }
    }
}

impl error::Error for MessageError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub struct MessageReceiver {
    domain_id: u16,
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
    spdp_data: SerializedPayload,
    disc_db: DiscoveryDB,
}

impl MessageReceiver {
    pub fn new(
        participant_guidprefix: GuidPrefix,
        domain_id: u16,
        disc_db: DiscoveryDB,
        wlp_timer_sender: mio_channel::Sender<EntityId>,
        spdp_data: SerializedPayload,
    ) -> MessageReceiver {
        Self {
            own_guid_prefix: participant_guidprefix,
            domain_id,
            source_version: ProtocolVersion::PROTOCOLVERSION,
            source_vendor_id: VendorId::VENDORID_UNKNOW,
            source_guid_prefix: GuidPrefix::UNKNOW,
            dest_guid_prefix: GuidPrefix::UNKNOW,
            unicast_reply_locator_list: vec![Locator::INVALID],
            multicast_reply_locator_list: vec![Locator::INVALID],
            wlp_timer_sender,
            have_timestamp: false,
            timestamp: Timestamp::TIME_INVALID,
            spdp_data,
            disc_db,
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
    ) -> Option<Vec<ReaderTimer>> {
        let mut rtv = Vec::new();
        for message in messages {
            let msg = message.message;
            if msg.len() < 16 {
                warn!("receive too small message: len = {}", msg.len());
                continue;
            }
            if msg[0..4] != b"RTPS"[..] {
                warn!("receive message not start with magic number 'RTPS'",);
                continue;
            }
            if msg.len() < 20 {
                // Is DDSPING
                if msg[9..16] == b"DDSPING"[..] {
                    trace!("Received DDSPING");
                    continue;
                } else {
                    warn!("receive too small message: len = {}", msg.len());
                    continue;
                }
            }
            let msg_buf = msg.freeze();
            let rtps_message = match Message::new(msg_buf) {
                Ok(m) => m,
                Err(e) => {
                    error!("failed to deserialize RTPS message: {}", e);
                    continue;
                }
            };
            trace!(
                "receive RTPS message form {} with {}",
                rtps_message.header.guid_prefix,
                rtps_message.summary()
            );
            if let Some(mut rt) = self.handle_parsed_packet(rtps_message, writers, readers) {
                rtv.append(&mut rt);
            };
        }
        if rtv.is_empty() {
            None
        } else {
            Some(rtv)
        }
    }

    fn handle_parsed_packet(
        &mut self,
        rtps_msg: Message,
        writers: &mut BTreeMap<EntityId, Writer>,
        readers: &mut BTreeMap<EntityId, Reader>,
    ) -> Option<Vec<ReaderTimer>> {
        self.reset();
        self.dest_guid_prefix = self.own_guid_prefix;
        self.source_guid_prefix = rtps_msg.header.guid_prefix;
        let mut rtv = Vec::new();
        for submsg in rtps_msg.submessages {
            match submsg.body {
                SubMessageBody::Entity(e) => {
                    match self.handle_entity_submessage(e, writers, readers) {
                        Ok(rt) => {
                            if let Some(mut rt) = rt {
                                rtv.append(&mut rt)
                            }
                        }
                        Err(MessageError::Error(e)) => {
                            error!("{}", e);
                            return if rtv.is_empty() { None } else { Some(rtv) };
                        }
                        Err(MessageError::Warn(w)) => {
                            warn!("{}", w);
                            return if rtv.is_empty() { None } else { Some(rtv) };
                        }
                    }
                }
                SubMessageBody::Interpreter(i) => match self.handle_interpreter_submessage(i) {
                    Ok(_) => {}
                    Err(MessageError::Error(e)) => {
                        error!("{}", e);
                        return if rtv.is_empty() { None } else { Some(rtv) };
                    }
                    Err(MessageError::Warn(w)) => {
                        warn!("{}", w);
                        return if rtv.is_empty() { None } else { Some(rtv) };
                    }
                },
            }
        }
        if rtv.is_empty() {
            None
        } else {
            Some(rtv)
        }
    }

    fn handle_entity_submessage(
        &mut self,
        entity_subm: EntitySubmessage,
        writers: &mut BTreeMap<EntityId, Writer>,
        readers: &mut BTreeMap<EntityId, Reader>,
    ) -> Result<Option<Vec<ReaderTimer>>, MessageError> {
        match entity_subm {
            EntitySubmessage::AckNack(acknack, flags) => self
                .handle_acknack_submsg(acknack, flags, writers)
                .map(|_| None),
            EntitySubmessage::Data(data, flags) => {
                self.handle_data_submsg(data, flags, readers, writers)
            }
            EntitySubmessage::DataFrag(data_frag, flags) => {
                Self::handle_datafrag_submsg(data_frag, flags).map(|_| None)
            }
            EntitySubmessage::Gap(gap, flags) => {
                self.handle_gap_submsg(gap, flags, readers).map(|_| None)
            }
            EntitySubmessage::HeartBeat(heartbeat, flags) => self
                .handle_heartbeat_submsg(heartbeat, flags, readers)
                .map(|_| None),
            EntitySubmessage::HeartbeatFrag(heartbeatfrag, flags) => {
                Self::handle_heartbeatfrag_submsg(heartbeatfrag, flags).map(|_| None)
            }
            EntitySubmessage::NackFrag(nack_frag, flags) => {
                Self::handle_nackfrag_submsg(nack_frag, flags).map(|_| None)
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
                        return Err(MessageError::Warn(
                            "Invalid InfoReply Submessage".to_string(),
                        ));
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
                        return Err(MessageError::Warn(
                            "Invalid InfoReplyIp4 Submessage".to_string(),
                        ));
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
                        return Err(MessageError::Warn(
                            "Invalid InfoTimestamp Submessage".to_string(),
                        ));
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
    ) -> Result<Option<WriterTimer>, MessageError> {
        // rtps 2.3 spec 8.3.7. AckNack

        if self.source_guid_prefix == self.own_guid_prefix
            && self.dest_guid_prefix != GuidPrefix::UNKNOW
        {
            trace!("message from same Participant & dest_guid_prefix is not UNKNOWN");
            return Ok(None);
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
            trace!("received Preemptive ACKNACK from Reader {}", reader_guid);
            return Ok(None);
        }

        if !ackanck.is_valid() {
            return Err(MessageError::Warn(format!(
                "Invalid AckNack Submessage from Reader {reader_guid}",
            )));
        }

        if let Some(w) = writers.get_mut(&ackanck.writer_id) {
            Ok(w.handle_acknack(ackanck, reader_guid))
        } else {
            Ok(None)
        }
    }
    fn handle_data_submsg(
        &mut self,
        data: Data,
        flag: BitFlags<DataFlag>,
        readers: &mut BTreeMap<EntityId, Reader>,
        writers: &mut BTreeMap<EntityId, Writer>,
    ) -> Result<Option<Vec<ReaderTimer>>, MessageError> {
        // rtps 2.3 spec 8.3.7.2 Data

        let mut rtv = Vec::new();

        if self.source_guid_prefix == self.own_guid_prefix
            && self.dest_guid_prefix != GuidPrefix::UNKNOW
        {
            trace!("message from same Participant & dest_guid_prefix is not UNKNOWN");
            if rtv.is_empty() {
                return Ok(None);
            } else {
                return Ok(Some(rtv));
            }
        }

        // validation
        if data.writer_sn < SequenceNumber(0)
            || data.writer_sn == SequenceNumber::SEQUENCENUMBER_UNKNOWN
        {
            return Err(MessageError::Warn("Invalid Data Submessage".to_string()));
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
            return Err(MessageError::Warn(
                "received DATA with inlineQos, Umber DDS dosen't support inlineQos yet".to_string(),
            ));
        }
        if flag.contains(DataFlag::NonStandardPayload) {
            // the serializedPayload element is not formatted according to Section 10.
            // This flag is informational. It indicates that the SerializedPayload has been transformed as described in another specification
            // For example, this flag should be set when the SerializedPayload is transformed as described in the DDS-Security specification
        }

        // rtps 2.3 spec, 8.3.7.2.5 Logical Interpretation
        let writer_guid = GUID::new(self.source_guid_prefix, data.writer_id);
        let _reader_guid = GUID::new(self.dest_guid_prefix, data.reader_id);

        let ts = Timestamp::now().expect("failed to get Timestamp::now()");
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
            self.handle_spdp_data(data, change, writers, readers)?;
        } else if data.writer_id == EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
            || data.reader_id == EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR
        {
            // if msg is for SEDP(w)
            self.handle_sedp_w_data(data, change, ts, readers)?;
        } else if data.writer_id == EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
            || data.reader_id == EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
        {
            // if msg is for SEDP(r)
            self.handle_sedp_r_data(data, change, writers, readers)?;
        } else if data.writer_id == EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER
            || data.reader_id == EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER
        {
            // if ParticipantMessage
            self.handle_participant_message(data, change, ts, readers)?;
        } else if data.reader_id == EntityId::UNKNOW {
            for reader in readers.values_mut() {
                if reader.is_contain_writer(writer_guid) {
                    if let Some(mut rt) = reader.add_change(self.source_guid_prefix, change.clone())
                    {
                        rtv.append(&mut rt);
                    };
                    self.disc_db.write_remote_writer(
                        writer_guid,
                        ts,
                        reader.get_matched_writer_qos(writer_guid).liveliness().kind,
                    );
                }
            }
        } else {
            match readers.get_mut(&data.reader_id) {
                Some(r) => {
                    if r.is_contain_writer(writer_guid) {
                        if let Some(mut rt) = r.add_change(self.source_guid_prefix, change) {
                            rtv.append(&mut rt);
                        };
                        self.disc_db.write_remote_writer(
                            writer_guid,
                            ts,
                            r.get_matched_writer_qos(writer_guid).liveliness().kind,
                        );
                    } else {
                        warn!(
                            "recieved DATA message from unknown Writer {}",
                            data.writer_id
                        );
                    }
                }
                None => {
                    warn!("recieved DATA message to unknown Reader {}", data.reader_id);
                }
            };
        }
        if rtv.is_empty() {
            Ok(None)
        } else {
            Ok(Some(rtv))
        }
    }
    fn handle_participant_discovery(
        &self,
        guid_prefix: GuidPrefix,
        spdp_data: SPDPdiscoveredParticipantData,
        writers: &mut BTreeMap<EntityId, Writer>,
        readers: &mut BTreeMap<EntityId, Reader>,
    ) {
        trace!("handle_participant_discovery: {}", guid_prefix);
        trace!("spdp_data: {:?}", spdp_data);
        if spdp_data.domain_id != self.domain_id {
        } else {
            let available_builtin_endpoint = spdp_data.available_builtin_endpoint;
            if available_builtin_endpoint
                .contains(BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
            {
                let guid = GUID::new(
                    spdp_data.guid.guid_prefix,
                    EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                );
                if let Some(writer) =
                    writers.get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)
                {
                    trace!("sedp_writer.matched_reader_add(remote_sedp_pub_reader)");
                    let qos = DataReaderQosBuilder::new()
                        .reliability(Reliability::default_besteffort())
                        .durability(Durability::TransientLocal)
                        .build();
                    writer.matched_reader_add(
                        guid,
                        spdp_data.expects_inline_qos,
                        spdp_data.metarraffic_unicast_locator_list.clone(),
                        spdp_data.metarraffic_multicast_locator_list.clone(),
                        qos,
                    );
                    // Executing heavy operations inside the event loop can severely impact
                    // performance, particularly under high load. It was identified that
                    // `min_message_cover()`, called via `Writer::send_heartbeat()`, was
                    // causing bottlenecks during periods of high activity.
                    //
                    // This specific `send_heartbeat()` call was an optimization intended to
                    // reduce the latency between Participant discovery and the first
                    // Heartbeat, but it is not essential for correct protocol operation.
                    // Removing this call improves event loop responsiveness without
                    // compromising reliability.
                    // writer.send_heart_beat(false);
                } else {
                    error!("not found writer 'SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER' from EventLoop.writers");
                }
            }
            if available_builtin_endpoint
                .contains(BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
            {
                let guid = GUID::new(
                    spdp_data.guid.guid_prefix,
                    EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                );
                if let Some(reader) = readers.get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR)
                {
                    trace!("sedp_reader.matched_writer_add(remote_sedp_pub_writer)");
                    let qos = DataWriterQosBuilder::new()
                        .reliability(Reliability::default_reliable())
                        .durability(Durability::TransientLocal)
                        .build();
                    reader.matched_writer_add(
                        guid,
                        spdp_data.metarraffic_unicast_locator_list.clone(),
                        spdp_data.metarraffic_multicast_locator_list.clone(),
                        0, // TODO: What value should I set?
                        qos,
                    );
                } else {
                    error!("not found reader 'SEDP_BUILTIN_PUBLICATIONS_DETECTOR' from EventLoop.readers");
                }
            }
            if available_builtin_endpoint
                .contains(BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
            {
                let guid = GUID::new(
                    spdp_data.guid.guid_prefix,
                    EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                );
                if let Some(writer) =
                    writers.get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER)
                {
                    trace!("sedp_writer.matched_reader_add(remote_sedp_sub_reader)");
                    let qos = DataReaderQosBuilder::new()
                        .reliability(Reliability::default_reliable())
                        .durability(Durability::TransientLocal)
                        .build();
                    writer.matched_reader_add(
                        guid,
                        spdp_data.expects_inline_qos,
                        spdp_data.metarraffic_unicast_locator_list.clone(),
                        spdp_data.metarraffic_multicast_locator_list.clone(),
                        qos,
                    );
                    // writer.send_heart_beat(false);
                } else {
                    error!("not found writer 'SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER' from EventLoop.writers");
                }
            }
            if available_builtin_endpoint
                .contains(BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
            {
                let guid = GUID::new(
                    spdp_data.guid.guid_prefix,
                    EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                );
                if let Some(reader) =
                    readers.get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR)
                {
                    trace!("sedp_reader.matched_writer_add(remote_sedp_sub_writer)");
                    let qos = DataWriterQosBuilder::new()
                        .reliability(Reliability::default_reliable())
                        .durability(Durability::TransientLocal)
                        .build();
                    reader.matched_writer_add(
                        guid,
                        spdp_data.metarraffic_unicast_locator_list.clone(),
                        spdp_data.metarraffic_multicast_locator_list.clone(),
                        0, // TODO: What value should I set?
                        qos,
                    );
                } else {
                    error!("not found reader 'SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR' from EventLoop.readers");
                }
            }
            if available_builtin_endpoint
                .contains(BuiltinEndpoint::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER)
            {
                let guid = GUID::new(
                    spdp_data.guid.guid_prefix,
                    EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
                );
                if let Some(writer) =
                    writers.get_mut(&EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER)
                {
                    trace!("p2p_msg_writer.matched_reader_add(remote_p2p_msg_reader)");
                    // rtps 2.3 sepc, 8.4.13.3 BuiltinParticipantMessageWriter and BuiltinParticipantMessageReader QoS
                    // If the ParticipantProxy::builtinEndpointQos is included in the SPDPdiscoveredParticipantData, then the
                    // BuiltinParticipantMessageWriter shall treat the BuiltinParticipantMessageReader as indicated by the flags If the
                    // ParticipantProxy::builtinEndpointQos is not included then the BuiltinParticipantMessageWriter shall treat the
                    // BuiltinParticipantMessageReader as if it is configured with RELIABLE_RELIABILITY_QOS.
                    let qos = DataReaderQosBuilder::new()
                        .reliability(Reliability::default_reliable()) // TODO: set best_effort if the flag indicated
                        .durability(Durability::TransientLocal)
                        .history(History {
                            kind: HistoryQosKind::KeepLast,
                            depth: 1,
                        })
                        .build();
                    writer.matched_reader_add(
                        guid,
                        spdp_data.expects_inline_qos,
                        spdp_data.metarraffic_unicast_locator_list.clone(),
                        spdp_data.metarraffic_multicast_locator_list.clone(),
                        qos,
                    );
                } else {
                    error!(
                                        "not found writer 'P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER' from EventLoop.writers"
                                    );
                }
            }
            if available_builtin_endpoint
                .contains(BuiltinEndpoint::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER)
            {
                let guid = GUID::new(
                    spdp_data.guid.guid_prefix,
                    EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
                );
                if let Some(reader) =
                    readers.get_mut(&EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER)
                {
                    trace!("p2p_msg_reader.matched_writer_add(remote_p2p_msg_writer)");
                    let qos = DataWriterQosBuilder::new()
                        .reliability(Reliability::default_reliable())
                        .durability(Durability::TransientLocal)
                        .history(History {
                            kind: HistoryQosKind::KeepLast,
                            depth: 1,
                        })
                        .build();
                    reader.matched_writer_add(
                        guid,
                        spdp_data.metarraffic_unicast_locator_list,
                        spdp_data.metarraffic_multicast_locator_list,
                        0, // TODO: What value should I set?
                        qos,
                    );
                } else {
                    error!("not found reader 'P2P_BUILTIN_PARTICIPANT_MESSAGE_READER' from EventLoop.readers");
                }
            }
        }
    }
    fn handle_spdp_data(
        &mut self,
        data: Data,
        _change: CacheChange,
        writers: &mut BTreeMap<EntityId, Writer>,
        readers: &mut BTreeMap<EntityId, Reader>,
    ) -> Result<(), MessageError> {
        let mut deserialized = if let Some(sp) = data.serialized_payload.as_ref() {
            let bytes = sp.to_bytes();
            let encapsulation_kind = RepresentationIdentifier::new([bytes[0], bytes[1]]);
            let _encapsulation_option = [bytes[2], bytes[3]];
            let endianness = match encapsulation_kind {
                RepresentationIdentifier::CDR_LE => Endianness::LittleEndian,
                RepresentationIdentifier::CDR_BE => Endianness::BigEndian,
                RepresentationIdentifier::PL_CDR_LE => Endianness::LittleEndian,
                RepresentationIdentifier::PL_CDR_BE => Endianness::BigEndian,
                rep => {
                    let bytes = rep.bytes();
                    panic!(
                        "unexpected encapsulation_kind: [0x{:02x}, 0x{:02x}]",
                        bytes[0], bytes[1]
                    )
                }
            };
            match SDPBuiltinData::read_from_buffer_with_ctx(endianness, &bytes[4..]) {
                Ok(data) => data,
                Err(e) => {
                    return Err(MessageError::Error(format!(
                        "failed to deserialize reseived spdp data message: {e:?}",
                    )));
                }
            }
        } else {
            return Err(MessageError::Warn(
                "received participant message without serializedPayload".to_string(),
            ));
        };
        let new_data = match deserialized.gen_spdp_discoverd_participant_data() {
            Some(nd) => nd,
            None => {
                return Err(MessageError::Warn(
                    "received incomplete spdp message".to_string(),
                ))
            }
        };
        if self.domain_id != new_data.domain_id {
            return Err(MessageError::Warn(format!(
                "received spdp message from different DDS domain. Self: {}, Remote: {}",
                self.domain_id, new_data.domain_id,
            )));
        }
        let guid_prefix = new_data.guid.guid_prefix;
        trace!("handle SPDP message from: {}", guid_prefix);
        let known = self.disc_db.write_participant(
            guid_prefix,
            Timestamp::now().unwrap_or(Timestamp::TIME_INVALID),
            new_data.clone(),
        );
        // TODO: if the participant is known, but parameters updated what to do?
        if !known {
            let locators = if !new_data.metarraffic_unicast_locator_list.is_empty() {
                new_data.metarraffic_unicast_locator_list.clone()
            } else if !new_data.default_unicast_locator_list.is_empty() {
                new_data.default_unicast_locator_list.clone()
            } else {
                Locator::new_list_from_multi_ipv4(
                    7400,
                    vec![std::net::Ipv4Addr::new(239, 255, 0, 1)],
                )
            };
            // spdp from unknown Participant
            match writers.get_mut(&EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER) {
                Some(w) => {
                    w.send_builtin_data_for_loc(self.spdp_data.clone(), locators);
                }
                None => {
                    error!("not found spdp_builtin_participant_writer");
                }
            }
        }
        self.handle_participant_discovery(guid_prefix, new_data, writers, readers);
        /*
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
        */
        Ok(())
    }
    fn handle_sedp_w_data(
        &mut self,
        data: Data,
        change: CacheChange,
        ts: Timestamp,
        readers: &mut BTreeMap<EntityId, Reader>,
    ) -> Result<(), MessageError> {
        let mut deserialized = if let Some(sp) = data.serialized_payload.as_ref() {
            let bytes = sp.to_bytes();
            let encapsulation_kind = RepresentationIdentifier::new([bytes[0], bytes[1]]);
            let _encapsulation_option = [bytes[2], bytes[3]];
            let endianness = match encapsulation_kind {
                RepresentationIdentifier::CDR_LE => Endianness::LittleEndian,
                RepresentationIdentifier::CDR_BE => Endianness::BigEndian,
                RepresentationIdentifier::PL_CDR_LE => Endianness::LittleEndian,
                RepresentationIdentifier::PL_CDR_BE => Endianness::BigEndian,
                rep => {
                    let bytes = rep.bytes();
                    panic!(
                        "unexpected encapsulation_kind: [0x{:02x}, 0x{:02x}]",
                        bytes[0], bytes[1]
                    )
                }
            };
            match SDPBuiltinData::read_from_buffer_with_ctx(endianness, &bytes[4..]) {
                Ok(data) => data,
                Err(e) => {
                    return Err(MessageError::Error(format!(
                        "failed to deserialize reseived sedp(w) data message: {e:?}",
                    )));
                }
            }
        } else {
            return Err(MessageError::Warn(
                "received sedp message without serializedPayload".to_string(),
            ));
        };
        let writer_proxy = if let Some(participant_data) =
            self.disc_db.read_participant_data(self.source_guid_prefix)
        {
            let default_unicast_locator_list = participant_data.default_unicast_locator_list;
            let default_multicast_locator_list = participant_data.default_multicast_locator_list;
            match deserialized.gen_writerproxy(
                Arc::new(RwLock::new(HistoryCache::new(HistoryCacheType::Dummy))),
                default_unicast_locator_list,
                default_multicast_locator_list,
            ) {
                Some(wp) => wp,
                None => {
                    return Err(MessageError::Warn(
                        "failed to generate writer_proxy form received DATA(w)".to_string(),
                    ));
                }
            }
        } else {
            return Err(MessageError::Warn(
                "received sedp(w) from unknown participant".to_string(),
            ));
        };
        let (topic_name, data_type) = match deserialized.topic_info() {
            Some((tn, dt)) => (tn, dt),
            None => {
                return Err(MessageError::Warn(
                    "falied to get topic_info from received DATA(w)".to_string(),
                ));
            }
        };
        self.disc_db.write_remote_writer(
            writer_proxy.remote_writer_guid,
            ts,
            writer_proxy.qos.liveliness().kind,
        );
        for (eid, reader) in readers.iter_mut() {
            if reader.is_writer_match(topic_name, data_type) {
                let remote_writer_topic_kind =
                    writer_proxy.remote_writer_guid.entity_id.topic_kind();
                if let Some(tk) = remote_writer_topic_kind {
                    if reader.topic_kind() != tk {
                        error!(
                                "Reader found Writer which has same topic_name: '{}' & data_type: '{}' , but not match topic_kind\n\tReader: {:?}\n\tWriter: {:?}",
                                topic_name, data_type,
                                reader.topic_kind(),
                                tk,
                            );
                        return Ok(());
                    }
                }
                trace!(
                    "Reader found matched Writer\n\tReader: {}\n\tWriter: {}",
                    eid,
                    writer_proxy.remote_writer_guid
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
                    .expect("failed to send data via channel 'wlp_timer_sender'");
            }
        }
        match readers.get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR) {
            Some(r) => r.add_change(self.source_guid_prefix, change),
            None => {
                return Err(MessageError::Error(
                    "not find sedp_builtin_publication_reader".to_string(),
                ));
            }
        };
        Ok(())
    }
    fn handle_sedp_r_data(
        &self,
        data: Data,
        change: CacheChange,
        writers: &mut BTreeMap<EntityId, Writer>,
        readers: &mut BTreeMap<EntityId, Reader>,
    ) -> Result<(), MessageError> {
        let mut deserialized = if let Some(sp) = data.serialized_payload.as_ref() {
            let bytes = sp.to_bytes();
            let encapsulation_kind = RepresentationIdentifier::new([bytes[0], bytes[1]]);
            let _encapsulation_option = [bytes[2], bytes[3]];
            let endianness = match encapsulation_kind {
                RepresentationIdentifier::CDR_LE => Endianness::LittleEndian,
                RepresentationIdentifier::CDR_BE => Endianness::BigEndian,
                RepresentationIdentifier::PL_CDR_LE => Endianness::LittleEndian,
                RepresentationIdentifier::PL_CDR_BE => Endianness::BigEndian,
                rep => {
                    let bytes = rep.bytes();
                    panic!(
                        "unexpected encapsulation_kind: [0x{:02x}, 0x{:02x}]",
                        bytes[0], bytes[1]
                    )
                }
            };
            match SDPBuiltinData::read_from_buffer_with_ctx(endianness, &bytes[4..]) {
                Ok(data) => data,
                Err(e) => {
                    return Err(MessageError::Error(format!(
                        "failed to deserialize reseived sedp(r) data message: {e:?}",
                    )));
                }
            }
        } else {
            return Err(MessageError::Warn(
                "received sedp message without serializedPayload".to_string(),
            ));
        };
        let reader_proxy = if let Some(participant_data) =
            self.disc_db.read_participant_data(self.source_guid_prefix)
        {
            let default_unicast_locator_list = participant_data.default_unicast_locator_list;
            let default_multicast_locator_list = participant_data.default_multicast_locator_list;
            match deserialized.gen_readerpoxy(
                Arc::new(RwLock::new(HistoryCache::new(HistoryCacheType::Dummy))),
                default_unicast_locator_list,
                default_multicast_locator_list,
            ) {
                Some(rp) => rp,
                None => {
                    return Err(MessageError::Warn(
                        "failed to generate reader_proxy form received DATA(r)".to_string(),
                    ));
                }
            }
        } else {
            return Err(MessageError::Warn(
                "received sedp(r) from unknown participant".to_string(),
            ));
        };
        let (topic_name, data_type) = match deserialized.topic_info() {
            Some((tn, dt)) => (tn, dt),
            None => {
                return Err(MessageError::Warn(
                    "falied to get topic_info from received DATA(r)".to_string(),
                ));
            }
        };
        for (eid, writer) in writers.iter_mut() {
            if writer.is_reader_match(topic_name, data_type) {
                let remote_reader_topic_kind =
                    reader_proxy.remote_reader_guid.entity_id.topic_kind();
                if let Some(tk) = remote_reader_topic_kind {
                    if writer.topic_kind() != tk {
                        error!(
                                "Writer found Reader which has same topic_name: '{}' & data_type: '{}' , but not match topic_kind\n\tWriter: {:?}\n\tReader: {:?}",
                                topic_name, data_type,
                                writer.topic_kind(),
                                tk,
                            );
                        return Ok(());
                    }
                }
                info!(
                    "Writer found matched Reader\n\tWriter: {}\n\tWriter: {}",
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
                return Err(MessageError::Error(
                    "not find sedp_builtin_subscription_reader".to_string(),
                ));
            }
        };
        Ok(())
    }
    fn handle_participant_message(
        &mut self,
        data: Data,
        change: CacheChange,
        ts: Timestamp,
        readers: &mut BTreeMap<EntityId, Reader>,
    ) -> Result<(), MessageError> {
        let deserialized = if let Some(sp) = data.serialized_payload.as_ref() {
            let bytes = sp.to_bytes();
            let encapsulation_kind = RepresentationIdentifier::new([bytes[0], bytes[1]]);
            let _encapsulation_option = [bytes[2], bytes[3]];
            let endianness = match encapsulation_kind {
                RepresentationIdentifier::CDR_LE => Endianness::LittleEndian,
                RepresentationIdentifier::CDR_BE => Endianness::BigEndian,
                RepresentationIdentifier::PL_CDR_LE => Endianness::LittleEndian,
                RepresentationIdentifier::PL_CDR_BE => Endianness::BigEndian,
                rep => {
                    let bytes = rep.bytes();
                    panic!(
                        "unexpected encapsulation_kind: [0x{:02x}, 0x{:02x}]",
                        bytes[0], bytes[1]
                    )
                }
            };
            match ParticipantMessageData::read_from_buffer_with_ctx(endianness, &bytes[4..]) {
                Ok(data) => data,
                Err(e) => {
                    return Err(MessageError::Error(format!(
                        "failed to deserialize reseived participant data message: {e:?}",
                    )));
                }
            }
        } else {
            return Err(MessageError::Warn(
                "received participant message without serializedPayload".to_string(),
            ));
        };
        if self
            .disc_db
            .read_participant_data(self.source_guid_prefix)
            .is_none()
        {
            return Err(MessageError::Warn(
                "received DATA(m) from unknown participant".to_string(),
            ));
        }
        match deserialized.kind {
            ParticipantMessageKind::AUTOMATIC_LIVELINESS_UPDATE => {
                trace!(
                        "receved DATA(m) with ParticipantMessageKind::AUTOMATIC_LIVELINESS_UPDATE from Participant: {}",
                         self.source_guid_prefix
                    );
                self.disc_db.update_liveliness_with_guid_prefix(
                    deserialized.guid_prefix,
                    ts,
                    LivelinessQosKind::Automatic,
                );
            }
            ParticipantMessageKind::MANUAL_LIVELINESS_UPDATE => {
                trace!(
                        "receved DATA(m) with ParticipantMessageKind::MANUAL_LIVELINESS_UPDATE from Participant: {}",
                         self.source_guid_prefix
                    );
                self.disc_db.update_liveliness_with_guid_prefix(
                    deserialized.guid_prefix,
                    ts,
                    LivelinessQosKind::ManualByParticipant,
                );
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
        match readers.get_mut(&EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER) {
            Some(r) => r.add_change(self.source_guid_prefix, change),
            None => {
                return Err(MessageError::Error(
                    "not find p2p_builtin_participant_reader".to_string(),
                ));
            }
        };
        Ok(())
    }
    fn handle_datafrag_submsg(
        _data_frag: DataFrag,
        _flag: BitFlags<DataFragFlag>,
    ) -> Result<(), MessageError> {
        Err(MessageError::Warn(
            "received DATA_FRAG submessage, Umber DDS dosen't support DATA_FRAG yet".to_string(),
        ))
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
            trace!("message from same Participant & dest_guid_prefix is not UNKNOWN");
            return Ok(());
        }

        let writer_guid = GUID::new(self.source_guid_prefix, gap.writer_id);
        let _reader_guid = GUID::new(self.dest_guid_prefix, gap.reader_id);

        // validation
        if !gap.is_valid(flag) {
            return Err(MessageError::Warn(format!(
                "Invalid Gap Submessage from Writer\n\tWriter: {writer_guid}",
            )));
        }

        if gap.reader_id == EntityId::UNKNOW {
            for reader in readers.values_mut() {
                if reader.is_contain_writer(GUID::new(self.source_guid_prefix, gap.writer_id)) {
                    reader.handle_gap(writer_guid, &gap);
                }
            }
        } else {
            match readers.get_mut(&gap.reader_id) {
                Some(r) => r.handle_gap(writer_guid, &gap),
                None => {
                    warn!("receved GAP message to unknown Reader {}", gap.reader_id);
                }
            };
        }
        Ok(())
    }
    fn handle_heartbeat_submsg(
        &mut self,
        heartbeat: Heartbeat,
        flag: BitFlags<HeartbeatFlag>,
        readers: &mut BTreeMap<EntityId, Reader>,
    ) -> Result<(), MessageError> {
        // rtps 2.3 spec 8.3.7.5 Heartbeat

        if self.source_guid_prefix == self.own_guid_prefix
            && self.dest_guid_prefix != GuidPrefix::UNKNOW
        {
            trace!("message from same Participant & dest_guid_prefix is not UNKNOWN");
            return Ok(());
        }

        let writer_guid = GUID::new(self.source_guid_prefix, heartbeat.writer_id);
        let _reader_guid = GUID::new(self.dest_guid_prefix, heartbeat.reader_id);

        // validation
        if !heartbeat.is_valid(flag) {
            return Err(MessageError::Warn(format!(
                "Invalid Heartbeat Submessage from Writer\n\tWriter: {writer_guid}",
            )));
        }

        let ts = Timestamp::now().expect("failed to get Timestamp::now()");

        macro_rules! update_liveliness_if_need {
            ($r: expr, $ts: expr) => {
                if flag.contains(HeartbeatFlag::Liveliness) {
                    self.disc_db.write_remote_writer(
                        writer_guid,
                        $ts,
                        $r.get_matched_writer_qos(writer_guid).liveliness().kind,
                    );
                }
            };
        }

        if heartbeat.reader_id == EntityId::UNKNOW {
            match heartbeat.writer_id {
                EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER => {
                    match readers.get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR) {
                        Some(r) => {
                            r.handle_heartbeat(writer_guid, flag, &heartbeat);
                            update_liveliness_if_need!(r, ts);
                        }
                        None => {
                            unreachable!("not found SEDP_BUILTIN_PUBLICATIONS_DETECTOR");
                        }
                    };
                }
                EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER => {
                    match readers.get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR) {
                        Some(r) => {
                            r.handle_heartbeat(writer_guid, flag, &heartbeat);
                            update_liveliness_if_need!(r, ts);
                        }
                        None => {
                            unreachable!("not found SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR");
                        }
                    };
                }
                writer_eid => {
                    for reader in readers.values_mut() {
                        if reader.is_contain_writer(GUID::new(self.source_guid_prefix, writer_eid))
                        {
                            reader.handle_heartbeat(writer_guid, flag, &heartbeat);
                            update_liveliness_if_need!(reader, ts);
                        }
                    }
                }
            }
        } else {
            match readers.get_mut(&heartbeat.reader_id) {
                Some(r) => {
                    r.handle_heartbeat(writer_guid, flag, &heartbeat);
                    update_liveliness_if_need!(r, ts);
                }
                None => {
                    warn!(
                        "receved HEARTBEAT message to unknown Reader {}",
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
        Err(MessageError::Warn(
            "received HEARTBEAT_FRAG submessage, Umber DDS dosen't support HEARTBEAT_FRAG yet"
                .to_string(),
        ))
    }
    fn handle_nackfrag_submsg(
        _nack_frag: NackFrag,
        _flag: BitFlags<NackFragFlag>,
    ) -> Result<(), MessageError> {
        Err(MessageError::Warn(
            "received NACK_FRAG submessage, Umber DDS dosen't support NACK_FRAG yet".to_string(),
        ))
    }
}
