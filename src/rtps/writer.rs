use crate::dds::{
    qos::{
        policy::{HistoryQosKind, ReliabilityQosKind},
        DataReaderQosPolicies, DataWriterQosPolicies,
    },
    Topic,
};
use crate::discovery::{
    structure::data::{DiscoveredWriterData, ParticipantMessageData, ParticipantMessageKind},
    ParticipantMessageCmd,
};
use crate::message::{
    message_builder::MessageBuilder,
    submessage::element::{AckNack, Count, Locator, SequenceNumber, SequenceNumberSet, Timestamp},
};
use crate::network::udp_sender::UdpSender;
use crate::rtps::cache::{ChangeForReaderStatusKind, HistoryCache, HistoryCacheType};
use crate::rtps::reader_locator::ReaderLocator;
use crate::structure::{
    Duration, EntityId, GuidPrefix, RTPSEntity, ReaderProxy, TopicKind, WriterProxy, GUID,
};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::rc::Rc;
use alloc::sync::Arc;
use awkernel_sync::rwlock::RwLock;
use core::net::Ipv4Addr;
use core::time::Duration as CoreDuration;
use log::{debug, error, info, warn};
use mio_extras::channel as mio_channel;
use mio_v06::Token;
use speedy::{Endianness, Writable};

/// RTPS StatefulWriter
pub struct Writer {
    // Entity
    guid: GUID,
    // Endpoint
    topic_kind: TopicKind,
    reliability_level: ReliabilityQosKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    // Writer
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    _nack_suppression_duration: Duration,
    writer_cache: Arc<RwLock<HistoryCache>>,
    data_max_size_serialized: i32,
    // StatelessWriter
    _reader_locators: Vec<ReaderLocator>,
    // StatefulWriter
    matched_readers: BTreeMap<GUID, ReaderProxy>,
    total_matched_readers: BTreeSet<GUID>,
    // This implementation spesific
    topic: Topic,
    qos: DataWriterQosPolicies,
    endianness: Endianness,
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
    writer_state_notifier: mio_channel::Sender<DataWriterStatusChanged>,
    set_writer_nack_sender: mio_channel::Sender<(EntityId, GUID)>,
    participant_msg_cmd_sender: mio_channel::SyncSender<ParticipantMessageCmd>,
    udp_sender: Rc<UdpSender>,
    hb_counter: Count,
    an_state: AckNackState,
    unmatch_count: i32,
    is_alive: bool,
}

#[derive(PartialEq, Eq)]
enum AckNackState {
    Waiting,
    Repairing,
    MustRepair,
}

impl Writer {
    pub fn new(
        wi: WriterIngredients,
        udp_sender: Rc<UdpSender>,
        set_writer_nack_sender: mio_channel::Sender<(EntityId, GUID)>,
    ) -> Self {
        let mut msg = String::new();
        msg += "\tunicast locators\n";
        for loc in &wi.unicast_locator_list {
            msg += &format!("\t\t{loc}\n");
        }
        msg += "\tmulticast locators\n";
        for loc in &wi.multicast_locator_list {
            msg += &format!("\t\t{loc}\n");
        }
        info!(
            "created new Writer of Topic ({}, {}) with Locators\n{}\tWriter: {}",
            wi.topic.name(),
            wi.topic.type_desc(),
            msg,
            wi.guid,
        );
        let writer_cache = wi.whc;
        writer_cache.write().add_empty_change(wi.guid);
        Self {
            guid: wi.guid,
            topic_kind: wi.topic.kind(),
            reliability_level: wi.reliability_level,
            unicast_locator_list: wi.unicast_locator_list,
            multicast_locator_list: wi.multicast_locator_list,
            push_mode: wi.push_mode,
            heartbeat_period: wi.heartbeat_period,
            nack_response_delay: wi.nack_response_delay,
            _nack_suppression_duration: wi.nack_suppression_duration,
            writer_cache,
            data_max_size_serialized: wi.data_max_size_serialized,
            _reader_locators: Vec::new(),
            matched_readers: BTreeMap::new(),
            total_matched_readers: BTreeSet::new(),
            topic: wi.topic,
            qos: wi.qos,
            endianness: Endianness::LittleEndian,
            writer_command_receiver: wi.writer_command_receiver,
            writer_state_notifier: wi.writer_state_notifier,
            set_writer_nack_sender,
            participant_msg_cmd_sender: wi.participant_msg_cmd_sender,
            udp_sender,
            hb_counter: 0,
            an_state: AckNackState::Waiting,
            unmatch_count: 0,
            is_alive: true,
        }
    }

    pub fn topic_kind(&self) -> TopicKind {
        self.topic_kind
    }

    pub fn sedp_data(&self) -> DiscoveredWriterData {
        let proxy = WriterProxy::new(
            self.guid,
            self.unicast_locator_list.clone(),
            self.multicast_locator_list.clone(),
            Vec::new(),
            Vec::new(),
            self.data_max_size_serialized,
            self.qos.clone(),
            Arc::new(RwLock::new(HistoryCache::new(HistoryCacheType::Dummy))),
        );
        let pub_data = self.topic.pub_builtin_topic_data();
        DiscoveredWriterData::new(proxy, pub_data)
    }

    pub fn is_reliable(&self) -> bool {
        match self.reliability_level {
            ReliabilityQosKind::Reliable => true,
            ReliabilityQosKind::BestEffort => false,
        }
    }

    pub fn _reader_locator_add(&mut self, locator: ReaderLocator) {
        // StatelessWriter
        self._reader_locators.push(locator)
    }

    pub fn _reader_locator_remove(&mut self, locator: ReaderLocator) {
        // StatelessWriter
        if let Some(loc_pos) = self._reader_locators.iter().position(|rl| *rl == locator) {
            self._reader_locators.remove(loc_pos);
        }
    }

    pub fn _unsent_changes_reset(&mut self) {
        // StatelessWriter
        for rl in &mut self._reader_locators {
            rl.unsent_changes = self.writer_cache.read().changes.values().cloned().collect();
        }
    }

    pub fn entity_token(&self) -> Token {
        self.entity_id().as_token()
    }

    pub fn handle_writer_cmd(&mut self) {
        while let Ok(cmd) = self.writer_command_receiver.try_recv() {
            match cmd {
                WriterCmd::WriteData => self.handle_write_data_cmd(),
                WriterCmd::AssertLiveliness => self.assert_liveliness_manually(),
            }
        }
    }

    pub fn assert_liveliness(&mut self) {
        self.is_alive = true;
        let ld = self.qos.liveliness().lease_duration;
        if ld == Duration::INFINITE {
            return;
        }
        let writer_cache = self.writer_cache.read();
        if let Some(ts) = writer_cache.get_last_added_ts(self.guid) {
            let elapse = Timestamp::now().expect("failed get Timestamp::new()") - *ts;
            if elapse > ld {
                let data = ParticipantMessageData::new(
                    self.guid_prefix(),
                    ParticipantMessageKind::AUTOMATIC_LIVELINESS_UPDATE,
                    vec![],
                );
                self.participant_msg_cmd_sender
                    .send(ParticipantMessageCmd::SendData(data))
                    .expect("couldn't send channel 'participant_msg_cmd_sender'");
            }
        } else {
            unreachable!(
                "Writer attempted to assert_liveliness, but writer_cache dosen't has last_added ts"
            );
        }
    }

    fn assert_liveliness_manually(&mut self) {
        self.is_alive = true;
        // rtps 2.3 spec, 8.3.7.5 Heartbeat, Table 8.38 - Structure of the Heartbeat Submessage
        // > LivelinessFlag: Appears in the Submessage header flags. Indicates that the DDS DataWriter associated with the RTPS Writer of the message has manually asserted its LIVELINESS.
        self.send_heart_beat(true);
    }

    fn handle_write_data_cmd(&mut self) {
        self.is_alive = true;
        // get changes from HistoryCache and register it to cache_state of ReaderProxy
        let history = self.qos.history();
        let hkind = history.kind;
        let hdepth = history.depth;
        let seq_nums = self.writer_cache.write().get_unprocessed();
        for (i, seq_num) in seq_nums.iter().rev().enumerate() {
            for reader_proxy in self.matched_readers.values_mut() {
                reader_proxy.update_cache_state(
                    *seq_num,
                    /* TODO: if DDS_FILTER(reader_proxy, change) { false } else { true }, */
                    match hkind {
                        HistoryQosKind::KeepAll => true,
                        HistoryQosKind::KeepLast => i + 1 >= hdepth as usize,
                    },
                    if self.push_mode {
                        ChangeForReaderStatusKind::Unsent
                    } else {
                        ChangeForReaderStatusKind::Unacknowledged
                    },
                )
            }
        }

        let self_guid = self.guid();
        let self_guid_prefix = self.guid_prefix();
        let self_entity_id = self.entity_id();
        let mut to_send_data: BTreeMap<SequenceNumber, Vec<(GUID, Vec<Locator>)>> = BTreeMap::new();
        let mut to_send_gap: BTreeMap<SequenceNumber, Vec<(GUID, Vec<Locator>)>> = BTreeMap::new();
        for reader_proxy in self.matched_readers.values_mut() {
            while let Some(change_for_reader) = reader_proxy.next_unsent_change() {
                reader_proxy.update_cache_state(
                    change_for_reader.seq_num,
                    change_for_reader.is_relevant,
                    ChangeForReaderStatusKind::Underway,
                );
                let seq_num = change_for_reader.seq_num;
                let reader_guid = reader_proxy.remote_reader_guid;
                let locator_list = if self_entity_id.is_builtin() {
                    // built-in endpoint send DATA via multicast
                    if let Some(ll_m) = Self::get_multicast_ll_from_proxy(self_guid, reader_proxy) {
                        ll_m
                    } else {
                        continue;
                    }
                } else {
                    // other endpoint send DATA via unicast
                    if let Some(ll_u) = Self::get_unicast_ll_from_proxy(self_guid, reader_proxy) {
                        ll_u
                    } else {
                        continue;
                    }
                };
                if change_for_reader.is_relevant {
                    if let std::collections::btree_map::Entry::Vacant(e) =
                        to_send_data.entry(seq_num)
                    {
                        e.insert(vec![(reader_guid, locator_list)]);
                    } else {
                        let vec = to_send_data.get_mut(&seq_num).unwrap();
                        vec.push((reader_guid, locator_list));
                    }
                } else if let std::collections::btree_map::Entry::Vacant(e) =
                    to_send_gap.entry(seq_num)
                {
                    e.insert(vec![(reader_guid, locator_list)]);
                } else {
                    let vec = to_send_gap.get_mut(&seq_num).unwrap();
                    vec.push((reader_guid, locator_list));
                }
            }
        }
        for seq_num in to_send_data.keys() {
            if let Some(aa_change) = self.writer_cache.read().get_change(self.guid, *seq_num) {
                let reader_locators = to_send_data.get(seq_num).unwrap();
                let send_list = Self::min_message_cover(reader_locators);
                for (reid, loc) in send_list {
                    // build RTPS Message
                    let mut message_builder = MessageBuilder::new();
                    let time_stamp = Timestamp::now();
                    message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                    message_builder.data(
                        Endianness::LittleEndian,
                        self.guid.entity_id,
                        reid,
                        aa_change,
                    );
                    let message = message_builder.build(self_guid_prefix);
                    let message_buf = message
                        .write_to_vec_with_ctx(self.endianness)
                        .expect("couldn't serialize message");
                    self.send_msg_to_locator(loc, message_buf, "data");
                }
            } else {
                unreachable!()
            }
        }
        for seq_num in to_send_gap.keys() {
            let reader_locators = to_send_gap.get(seq_num).unwrap();
            let send_list = Self::min_message_cover(reader_locators);
            for (reid, loc) in send_list {
                let mut message_builder = MessageBuilder::new();
                // let time_stamp = Timestamp::now();
                // message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                let gap_start = *seq_num;
                message_builder.gap(
                    Endianness::LittleEndian,
                    self.guid.entity_id,
                    reid,
                    gap_start,
                    SequenceNumberSet::from_vec(gap_start + SequenceNumber(1), vec![]),
                );
                let message = message_builder.build(self_guid_prefix);
                let message_buf = message
                    .write_to_vec_with_ctx(self.endianness)
                    .expect("couldn't serialize message");
                self.send_msg_to_locator(loc, message_buf, "gap");
            }
        }
    }

    pub fn send_heart_beat(&mut self, liveliness: bool) {
        if liveliness {
            self.is_alive = true;
        } else if self.is_acked_by_all() {
            return;
        }
        let time_stamp = Timestamp::now();
        let writer_cache = self.writer_cache.read();
        let first_sn = writer_cache.get_seq_num_min();
        let last_sn = writer_cache.get_seq_num_max();
        if first_sn.0 <= 0 || last_sn.0 < 0 || last_sn.0 < first_sn.0 - 1 {
            debug!(
                "heartbeat validation failed\n\tfirstSN: {}, lastSN: {}",
                first_sn.0, last_sn.0
            );
            return;
        }
        self.hb_counter += 1;
        let self_guid = self.guid();
        let self_guid_prefix = self.guid_prefix();
        let self_entity_id = self.entity_id();
        let mut to_send_hb: Vec<(GUID, Vec<Locator>)> = Vec::new();
        for reader_proxy in self.matched_readers.values_mut() {
            let ll_m =
                if let Some(ll_m) = Self::get_multicast_ll_from_proxy(self_guid, reader_proxy) {
                    ll_m
                } else {
                    continue;
                };
            to_send_hb.push((reader_proxy.remote_reader_guid, ll_m));
        }
        let reader_locators = to_send_hb;
        let send_list = Self::min_message_cover(&reader_locators);
        for (reid, loc) in send_list {
            // build RTPS Message
            let mut message_builder = MessageBuilder::new();
            message_builder.info_ts(Endianness::LittleEndian, time_stamp);
            message_builder.heartbeat(
                self.endianness,
                liveliness,
                self_entity_id,
                reid,
                first_sn,
                last_sn,
                self.hb_counter - 1,
                false,
            );
            let msg = message_builder.build(self_guid_prefix);
            let message_buf = msg
                .write_to_vec_with_ctx(self.endianness)
                .expect("couldn't serialize message");
            self.send_msg_to_locator(loc, message_buf, "heartbeat");
        }
    }

    pub fn handle_acknack(&mut self, acknack: AckNack, reader_guid: GUID) {
        if let Some(reader_proxy) = self.matched_readers.get_mut(&reader_guid) {
            let req_seq_num_set = acknack.reader_sn_state.set();
            info!(
                "Writer handle acknack from Reader\n\tSNState: {}, Set: {:?}\n\tWriter: {}\n\tReader: {}",
                acknack.reader_sn_state.base().0,
                req_seq_num_set,
                self.guid, reader_guid
            );
            reader_proxy.acked_changes_set(acknack.reader_sn_state.base() - SequenceNumber(1));
            reader_proxy.requested_changes_set(req_seq_num_set);
            match self.an_state {
                AckNackState::Waiting => {
                    // Transistion T9
                    if !reader_proxy.requested_changes().is_empty() {
                        if self.nack_response_delay == Duration::ZERO {
                            // Transistion T11
                            self.handle_nack_response_timeout(reader_guid);
                        } else {
                            self.set_writer_nack_sender
                                .send((self.entity_id(), reader_guid))
                                .expect("couldn't send channel 'set_writer_nack_sender'");
                            self.an_state = AckNackState::MustRepair
                        }
                    }
                }
                AckNackState::MustRepair => {
                    // Transistion T10
                }
                AckNackState::Repairing => unreachable!(),
            }
        } else {
            warn!(
                "Writer tried handle ACKNACK from unmatched Reader\n\tWriter: {}\n\tReader: {}",
                self.guid, reader_guid
            );
        }
    }

    pub fn handle_nack_response_timeout(&mut self, reader_guid: GUID) {
        self.an_state = AckNackState::Repairing;
        let self_guid = self.guid();
        let self_guid_prefix = self.guid_prefix();
        let self_entity_id = self.entity_id();
        let mut to_send_data: BTreeMap<SequenceNumber, Vec<(GUID, Vec<Locator>)>> = BTreeMap::new();
        let mut to_send_gap: BTreeMap<SequenceNumber, Vec<(GUID, Vec<Locator>)>> = BTreeMap::new();
        if let Some(reader_proxy) = self.matched_readers.get_mut(&reader_guid) {
            while let Some(change) = reader_proxy.next_requested_change() {
                reader_proxy.update_cache_state(
                    change.seq_num,
                    change.is_relevant,
                    ChangeForReaderStatusKind::Underway,
                );
                let seq_num = change.seq_num;
                let reader_guid = reader_proxy.remote_reader_guid;
                let locator_list = if self_entity_id.is_builtin() {
                    // built-in endpoint send DATA via multicast
                    if let Some(ll_m) = Self::get_multicast_ll_from_proxy(self_guid, reader_proxy) {
                        ll_m
                    } else {
                        continue;
                    }
                } else {
                    // other endpoint send DATA via unicast
                    if let Some(ll_u) = Self::get_unicast_ll_from_proxy(self_guid, reader_proxy) {
                        ll_u
                    } else {
                        continue;
                    }
                };
                if change.is_relevant {
                    if let std::collections::btree_map::Entry::Vacant(e) =
                        to_send_data.entry(seq_num)
                    {
                        e.insert(vec![(reader_guid, locator_list)]);
                    } else {
                        let vec = to_send_data.get_mut(&seq_num).unwrap();
                        vec.push((reader_guid, locator_list));
                    }
                } else if let std::collections::btree_map::Entry::Vacant(e) =
                    to_send_gap.entry(seq_num)
                {
                    e.insert(vec![(reader_guid, locator_list)]);
                } else {
                    let vec = to_send_gap.get_mut(&seq_num).unwrap();
                    vec.push((reader_guid, locator_list));
                }
            }
        } else {
            error!(
                "not found Reader which attempt to send nack response from Writer\n\tReader: {}\n\tWriter: {}",
                reader_guid,  self.guid
            );
        }
        for seq_num in to_send_data.keys() {
            let reader_locators = to_send_data.get(seq_num).unwrap();
            let send_list = Self::min_message_cover(reader_locators);
            if let Some(aa_change) = self.writer_cache.read().get_change(self.guid, *seq_num) {
                for (reid, loc) in send_list {
                    // build RTPS Message
                    let mut message_builder = MessageBuilder::new();
                    let time_stamp = Timestamp::now();
                    message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                    message_builder.data(
                        Endianness::LittleEndian,
                        self.guid.entity_id,
                        reid,
                        aa_change,
                    );
                    let message = message_builder.build(self_guid_prefix);
                    let message_buf = message
                        .write_to_vec_with_ctx(self.endianness)
                        .expect("couldn't serialize message");
                    self.send_msg_to_locator(loc, message_buf, "data");
                }
            } else {
                // rtps spec, 8.4.2.2.4 Writers must eventually respond to a negative acknowledgment (reliable only)
                // send Heartbeat because, the sample is no longer avilable
                self.hb_counter += 1;
                let time_stamp = Timestamp::now();
                let writer_cache = self.writer_cache.read();
                for (reid, loc) in send_list {
                    let mut message_builder = MessageBuilder::new();
                    message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                    message_builder.heartbeat(
                        self.endianness,
                        false,
                        self_entity_id,
                        reid,
                        *seq_num + SequenceNumber(1),
                        writer_cache.get_seq_num_max(),
                        self.hb_counter - 1,
                        false,
                    );
                    let msg = message_builder.build(self_guid_prefix);
                    let message_buf = msg
                        .write_to_vec_with_ctx(self.endianness)
                        .expect("couldn't serialize message");
                    self.send_msg_to_locator(loc, message_buf, "heartbeat");
                }
            }
        }
        for seq_num in to_send_gap.keys() {
            let reader_locators = to_send_gap.get(seq_num).unwrap();
            let send_list = Self::min_message_cover(reader_locators);
            for (reid, loc) in send_list {
                let mut message_builder = MessageBuilder::new();
                // let time_stamp = Timestamp::now();
                // message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                let gap_start = *seq_num;
                message_builder.gap(
                    Endianness::LittleEndian,
                    self.guid.entity_id,
                    reid,
                    gap_start,
                    SequenceNumberSet::from_vec(gap_start + SequenceNumber(1), vec![]),
                );
                let message = message_builder.build(self_guid_prefix);
                let message_buf = message
                    .write_to_vec_with_ctx(self.endianness)
                    .expect("couldn't serialize message");
                self.send_msg_to_locator(loc, message_buf, "gap");
            }
        }
        self.an_state = AckNackState::Waiting;
    }

    fn send_msg_to_locator(&self, loc: Locator, msg_buf: Vec<u8>, msg_kind: &str) {
        if loc.kind == Locator::KIND_UDPV4 {
            let port = loc.port;
            let addr = loc.address;
            info!(
                "Writer send {} message to {}.{}.{}.{}:{}\n\tWriter: {}",
                msg_kind, addr[12], addr[13], addr[14], addr[15], port, self.guid,
            );
            if Self::is_ipv4_multicast(&addr) {
                self.udp_sender.send_to_multicast(
                    &msg_buf,
                    Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                    port as u16,
                );
            } else {
                self.udp_sender.send_to_unicast(
                    &msg_buf,
                    Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                    port as u16,
                );
            }
        } else {
            error!("unsupported locator specified: {}", loc);
        }
    }

    fn get_unicast_ll_from_proxy(
        my_guid: GUID,
        reader_proxy: &ReaderProxy,
    ) -> Option<Vec<Locator>> {
        let ll_u = reader_proxy.get_unicast_locator_list().clone();
        if ll_u.is_empty() {
            let ll_m = reader_proxy.get_multicast_locator_list().clone();
            if ll_m.is_empty() {
                error!(
                    "Writer not found locator of Reader\n\tWriter: {}\n\tReader: {}",
                    my_guid, reader_proxy.remote_reader_guid
                );
                None
            } else {
                debug!("Writer attempted to get unicast locators from the ReaderProxy, but not found. use multicast locators instead\n\tWriter: {}\n\tReader: {}", my_guid, reader_proxy.remote_reader_guid);
                Some(ll_m)
            }
        } else {
            Some(ll_u)
        }
    }

    fn get_multicast_ll_from_proxy(
        my_guid: GUID,
        reader_proxy: &ReaderProxy,
    ) -> Option<Vec<Locator>> {
        let ll_m = reader_proxy.get_multicast_locator_list().clone();
        if ll_m.is_empty() {
            let ll_u = reader_proxy.get_unicast_locator_list().clone();
            if ll_u.is_empty() {
                error!(
                    "Writer not found locator of Reader\n\tWriter: {}\n\tReader: {}",
                    my_guid, reader_proxy.remote_reader_guid
                );
                None
            } else {
                debug!(
                    "Writer attempted to get multicast locators from the ReaderProxy, but not found. use unicast locators instead\n\tWriter: {}\n\tReader: {}",
                    my_guid, reader_proxy.remote_reader_guid
                );
                Some(ll_u)
            }
        } else {
            Some(ll_m)
        }
    }

    pub fn is_acked_by_all(&self) -> bool {
        for reader_proxy in self.matched_readers.values() {
            if !reader_proxy.is_acked() {
                return false;
            }
        }
        true
    }

    fn is_ipv4_multicast(ipv4_addr: &[u8; 16]) -> bool {
        // 224.0.0.0 - 239.255.255.255
        ((ipv4_addr[12] >> 4) ^ 0b1110) == 0
    }

    fn min_message_cover(reader_locators: &Vec<(GUID, Vec<Locator>)>) -> Vec<(EntityId, Locator)> {
        let mut locator_to_readers: BTreeMap<Locator, BTreeSet<GUID>> = BTreeMap::new();
        for (guid, ll) in reader_locators {
            for l in ll {
                locator_to_readers.entry(*l).or_default().insert(*guid);
            }
        }
        let mut send_list = Vec::new();
        let mut covered: BTreeSet<(GUID, Locator)> = BTreeSet::new();
        for (loc, readers) in &locator_to_readers {
            if readers.len() > 1 {
                send_list.push((EntityId::UNKNOW, *loc));
                for r in readers {
                    covered.insert((*r, *loc));
                }
            }
        }
        for (reader, locs) in reader_locators {
            for loc in locs {
                if !covered.contains(&(*reader, *loc)) {
                    send_list.push((reader.entity_id, *loc));
                }
            }
        }
        send_list
    }

    pub fn matched_reader_add(
        &mut self,
        remote_reader_guid: GUID,
        expects_inline_qos: bool,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        qos: DataReaderQosPolicies,
    ) {
        self.matched_reader_add_with_default_locator(
            remote_reader_guid,
            expects_inline_qos,
            unicast_locator_list,
            multicast_locator_list,
            Vec::new(),
            Vec::new(),
            qos,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn matched_reader_add_with_default_locator(
        &mut self,
        remote_reader_guid: GUID,
        expects_inline_qos: bool,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        qos: DataReaderQosPolicies,
    ) {
        if let std::collections::btree_map::Entry::Vacant(e) =
            self.matched_readers.entry(remote_reader_guid)
        {
            if let Err(e) = self.qos.is_compatible(&qos) {
                self.writer_state_notifier
                    .send(DataWriterStatusChanged::OfferedIncompatibleQos(e.clone()))
                    .expect("couldn't send writer_state_notifier");
                warn!(
                "Writer offered incompatible qos from Reader\n\tWriter: {}\n\tReader: {}\n\terror: {}",
                self.guid, remote_reader_guid, e
            );
                return;
            }

            info!(
                "Writer found matched Reader\n\tWriter: {}\n\tReader: {}",
                self.guid, remote_reader_guid
            );

            e.insert(ReaderProxy::new(
                remote_reader_guid,
                expects_inline_qos,
                unicast_locator_list,
                multicast_locator_list,
                default_unicast_locator_list,
                default_multicast_locator_list,
                qos,
                self.writer_cache.clone(),
                self.push_mode,
            ));
            self.total_matched_readers.insert(remote_reader_guid);
            let pub_match_state = PublicationMatchedStatus::new(
                self.total_matched_readers.len() as i32,
                1,
                self.matched_readers.len() as i32,
                1,
                remote_reader_guid,
            );
            self.writer_state_notifier
                .send(DataWriterStatusChanged::PublicationMatched(pub_match_state))
                .expect("couldn't send writer_state_notifier");
        } else {
            let remote_reader = self.matched_readers.get_mut(&remote_reader_guid).unwrap();
            macro_rules! update_proxy_if_need {
                ($name:ident) => {
                    if remote_reader.$name != $name {
                        remote_reader.$name = $name;
                        info!(
                            "Writer update matched Reader info\n\tWriter: {}\n\tReader: {}",
                            self.guid, remote_reader_guid
                        );
                    }
                };
            }
            update_proxy_if_need!(qos);
            update_proxy_if_need!(expects_inline_qos);
            update_proxy_if_need!(unicast_locator_list);
            update_proxy_if_need!(multicast_locator_list);
            update_proxy_if_need!(default_unicast_locator_list);
            update_proxy_if_need!(default_multicast_locator_list);
        }
    }
    pub fn is_reader_match(&self, topic_name: &str, data_type: &str) -> bool {
        self.topic.name() == topic_name && self.topic.type_desc() == data_type
    }

    /*
    pub fn matched_reader_lookup(&self, guid: GUID) -> Option<ReaderProxy> {
        self.matched_readers.get(&guid).cloned()
    }
    */

    pub fn check_liveliness(&mut self) {
        if self.is_alive {
            let ld = self.qos.liveliness().lease_duration;
            if ld == Duration::INFINITE {
                return;
            }
            let writer_cache = self.writer_cache.read();
            if let Some(ts) = writer_cache.get_last_added_ts(self.guid) {
                let elapse = Timestamp::now().expect("failed get Timestamp::new()") - *ts;
                if elapse > ld {
                    self.unmatch_count += 1;
                    self.is_alive = false;
                    debug!("checked liveliness of local wirter Lost, ld: {:?}, elapse: {:?}\n\tWriter: {}", ld, elapse, self.guid.entity_id);
                    self.writer_state_notifier
                        .send(DataWriterStatusChanged::LivelinessLost(
                            LivelinessLostStatus::new(self.unmatch_count, 1, self.guid),
                        ))
                        .expect("couldn't send writer_state_notifier");
                }
            }
        }
    }

    fn _matched_reader_unmatch(&mut self, guid: GUID) {
        if self.matched_readers.remove(&guid).is_some() {
            self.unmatch_count += 1;
            self.writer_state_notifier
                .send(DataWriterStatusChanged::LivelinessLost(
                    LivelinessLostStatus::new(self.unmatch_count, 1, guid),
                ))
                .expect("couldn't send writer_state_notifier");
        }
    }

    fn matched_reader_remove(&mut self, guid: GUID) {
        self.matched_readers.remove(&guid);
        let pub_match_state = PublicationMatchedStatus::new(
            self.total_matched_readers.len() as i32,
            0,
            self.matched_readers.len() as i32,
            -1,
            guid,
        );
        self.writer_state_notifier
            .send(DataWriterStatusChanged::PublicationMatched(pub_match_state))
            .expect("couldn't send writer_state_notifier");
    }

    pub fn delete_reader_proxy(&mut self, guid_prefix: GuidPrefix) {
        let to_delete: Vec<GUID> = self
            .matched_readers
            .keys()
            .filter(|k| k.guid_prefix == guid_prefix)
            .copied()
            .collect();

        for d in to_delete {
            self.matched_reader_remove(d);
        }
    }

    pub fn heartbeat_period(&self) -> CoreDuration {
        CoreDuration::new(
            self.heartbeat_period.seconds as u64,
            self.heartbeat_period.fraction,
        )
    }

    pub fn nack_response_delay(&self) -> CoreDuration {
        CoreDuration::new(
            self.nack_response_delay.seconds as u64,
            self.nack_response_delay.fraction,
        )
    }

    pub fn get_qos(&self) -> DataWriterQosPolicies {
        self.qos.clone()
    }
}

impl RTPSEntity for Writer {
    fn guid(&self) -> GUID {
        self.guid
    }
}

/// For more details on each variants, please refer to the DDS specification. DDS v1.4 spec, 2.2.4 Listeners, Conditions, and Wait-sets (<https://www.omg.org/spec/DDS/1.4/PDF#G5.1034386>)
///
/// The content for each variant has not been implemented yet, but it is planned to be implemented in the future.
pub enum DataWriterStatusChanged {
    LivelinessLost(LivelinessLostStatus),
    OfferedDeadlineMissed,
    OfferedIncompatibleQos(String),
    PublicationMatched(PublicationMatchedStatus),
}

pub struct LivelinessLostStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    /// This is diffarent form DDS spec.
    /// The GUID is remote reader's one.
    pub guid: GUID,
}
impl LivelinessLostStatus {
    pub fn new(total_count: i32, total_count_change: i32, guid: GUID) -> Self {
        Self {
            total_count,
            total_count_change,
            guid,
        }
    }
}

pub struct PublicationMatchedStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub current_count: i32,
    pub current_count_change: i32,
    /// This is diffarent form DDS spec.
    /// The GUID is remote reader's one.
    pub guid: GUID,
}
impl PublicationMatchedStatus {
    pub fn new(
        total_count: i32,
        total_count_change: i32,
        current_count: i32,
        current_count_change: i32,
        guid: GUID,
    ) -> Self {
        Self {
            total_count,
            total_count_change,
            current_count,
            current_count_change,
            guid,
        }
    }
}

pub struct WriterIngredients {
    // Entity
    pub guid: GUID,
    // Endpoint
    pub reliability_level: ReliabilityQosKind,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    // Writer
    pub push_mode: bool,
    pub heartbeat_period: Duration,
    pub nack_response_delay: Duration,
    pub nack_suppression_duration: Duration,
    pub data_max_size_serialized: i32,
    pub(crate) whc: Arc<RwLock<HistoryCache>>,
    // This implementation spesific
    pub topic: Topic,
    pub qos: DataWriterQosPolicies,
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
    pub writer_state_notifier: mio_channel::Sender<DataWriterStatusChanged>,
    pub participant_msg_cmd_sender: mio_channel::SyncSender<ParticipantMessageCmd>,
}
pub enum WriterCmd {
    WriteData,
    AssertLiveliness,
}
