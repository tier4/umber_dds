use crate::dds::{
    qos::{policy::ReliabilityQosKind, DataReaderQosPolicies, DataWriterQosPolicies},
    Topic,
};
use crate::discovery::{
    discovery_db::{DiscoveryDB, EndpointState},
    structure::data::DiscoveredReaderData,
};
use crate::message::message_builder::MessageBuilder;
use crate::message::submessage::{
    element::{Gap, Heartbeat, Locator, SequenceNumber, SequenceNumberSet, Timestamp},
    submessage_flag::HeartbeatFlag,
};
use crate::network::udp_sender::UdpSender;
use crate::rtps::cache::{CacheChange, HistoryCache};
use crate::structure::{
    Duration, EntityId, GuidPrefix, RTPSEntity, ReaderProxy, TopicKind, WriterProxy, GUID,
};
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::sync::Arc;
use awkernel_sync::rwlock::RwLock;
use core::net::Ipv4Addr;
use core::time::Duration as StdDuration;
use enumflags2::BitFlags;
use log::{debug, error, info, trace, warn};
use mio_extras::channel as mio_channel;
use speedy::{Endianness, Writable};

/// RTPS StatefulReader
pub struct Reader {
    // Entity
    guid: GUID,
    // Endpoint
    topic_kind: TopicKind,
    reliability_level: ReliabilityQosKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    // Reader
    expectsinline_qos: bool,
    heartbeat_response_delay: Duration,
    reader_cache: Arc<RwLock<HistoryCache>>,
    // StatefulReader
    matched_writers: BTreeMap<GUID, WriterProxy>,
    unmatched_writers: BTreeMap<GUID, WriterProxy>,
    // This implementation spesific
    topic: Topic,
    qos: DataReaderQosPolicies,
    endianness: Endianness,
    reader_state_notifier: mio_channel::Sender<DataReaderStatusChanged>,
    set_reader_hb_timer_sender: mio_channel::Sender<(EntityId, GUID)>,
    udp_sender: Rc<UdpSender>,
}

impl Reader {
    pub fn new(
        ri: ReaderIngredients,
        udp_sender: Rc<UdpSender>,
        set_reader_hb_timer_sender: mio_channel::Sender<(EntityId, GUID)>,
    ) -> Self {
        let mut msg = String::new();
        msg += "\tunicast locators\n";
        for loc in &ri.unicast_locator_list {
            msg += &format!("\t\t{loc}\n");
        }
        msg += "\tmulticast locators\n";
        for loc in &ri.multicast_locator_list {
            msg += &format!("\t\t{loc}\n");
        }
        info!(
            "created new Reader of Topic ({}, {}) with Locators\n{}\tReader: {}",
            ri.topic.name(),
            ri.topic.type_desc(),
            msg,
            ri.guid,
        );
        Self {
            guid: ri.guid,
            topic_kind: ri.topic.kind(),
            reliability_level: ri.reliability_level,
            unicast_locator_list: ri.unicast_locator_list,
            multicast_locator_list: ri.multicast_locator_list,
            expectsinline_qos: ri.expectsinline_qos,
            heartbeat_response_delay: ri.heartbeat_response_delay,
            reader_cache: ri.rhc,
            matched_writers: BTreeMap::new(),
            unmatched_writers: BTreeMap::new(),
            topic: ri.topic,
            qos: ri.qos,
            endianness: Endianness::LittleEndian,
            reader_state_notifier: ri.reader_state_notifier,
            set_reader_hb_timer_sender,
            udp_sender,
        }
    }

    pub fn is_reliable(&self) -> bool {
        match self.reliability_level {
            ReliabilityQosKind::Reliable => true,
            ReliabilityQosKind::BestEffort => false,
        }
    }

    pub fn topic_kind(&self) -> TopicKind {
        self.topic_kind
    }

    pub fn sedp_data(&self) -> DiscoveredReaderData {
        let proxy = ReaderProxy::new(
            self.guid,
            self.expectsinline_qos,
            self.unicast_locator_list.clone(),
            self.multicast_locator_list.clone(),
            Vec::new(),
            Vec::new(),
            self.qos.clone(),
            Arc::new(RwLock::new(HistoryCache::new())),
        );
        let sub_data = self.topic.sub_builtin_topic_data();
        DiscoveredReaderData::new(proxy, sub_data)
    }

    pub fn add_change(&mut self, source_guid_prefix: GuidPrefix, change: CacheChange) {
        let writer_guid = GUID::new(source_guid_prefix, change.writer_guid.entity_id);
        if let Some(wp) = self.unmatched_writers.remove(&writer_guid) {
            self.matched_writers.insert(writer_guid, wp);
            self.reader_state_notifier
                .send(DataReaderStatusChanged::LivelinessChanged(
                    LivelinessChangedStatus::new(
                        self.matched_writers.len() as i32,
                        self.unmatched_writers.len() as i32,
                        1,
                        -1,
                        writer_guid,
                    ),
                ))
                .expect("couldn't send channel 'reader_state_notifier'");
        }
        info!(
            "Reader add change from Writer, seq_num: {}\n\tReader: {}\n\tWriter: {}",
            change.sequence_number.0, self.guid, writer_guid
        );
        if self.is_reliable() {
            // Reliable Reader Behavior
            if let Err(e) = self.reader_cache.write().add_change(change.clone()) {
                debug!(
                    "add_change to Reader failed: {}\n\tReader: {}\n\tWriter: {}",
                    e, self.guid, change.writer_guid
                );
                return;
            }
            self.reader_state_notifier
                .send(DataReaderStatusChanged::DataAvailable)
                .expect("couldn't send channel 'reader_state_notifier'");
            if let Some(writer_proxy) = self.matched_writers.get_mut(&writer_guid) {
                writer_proxy.received_chage_set(change.sequence_number);
            }
        } else {
            // BestEffort Reader Behavior
            if self.matched_writers.contains_key(&writer_guid) {
                let flag;
                let expected_seq_num;
                {
                    let writer_proxy = self
                        .matched_writers
                        .get(&writer_guid)
                        .expect("couldn't get writer_proxy");
                    expected_seq_num = writer_proxy.available_changes_max() + SequenceNumber(1);
                    flag = change.sequence_number >= expected_seq_num;
                }
                if flag {
                    if let Err(e) = self.reader_cache.write().add_change(change.clone()) {
                        info!(
                            "add_change to Reader failed: {}\n\tReader: {}\n\tWriter: {}",
                            e, self.guid, change.writer_guid
                        );
                        return;
                    }
                    self.reader_state_notifier
                        .send(DataReaderStatusChanged::DataAvailable)
                        .expect("couldn't send reader_state_notifier");
                    let writer_proxy_mut = self
                        .matched_writers
                        .get_mut(&writer_guid)
                        .expect("couldn't get writer_proxy_mut");
                    writer_proxy_mut.received_chage_set(change.sequence_number);
                    if change.sequence_number > expected_seq_num {
                        writer_proxy_mut.lost_changes_update(change.sequence_number);
                    }
                } else {
                    debug!("BestEffort Reader receive change whose sequence_number < expected_seq_num\n\tReader: {}\n\tWriter: {}", self.guid, writer_guid);
                }
            } else {
                warn!(
                    "BestEffort Reader tried add change from unmatched Writer\n\tReader: {}\n\tWriter: {}",
                    self.guid, writer_guid
                );
            }
        }
    }

    pub fn matched_writer_add(
        &mut self,
        remote_writer_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: i32,
        qos: DataWriterQosPolicies,
    ) {
        self.matched_writer_add_with_default_locator(
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
            Vec::new(),
            Vec::new(),
            data_max_size_serialized,
            qos,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn matched_writer_add_with_default_locator(
        &mut self,
        remote_writer_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: i32,
        qos: DataWriterQosPolicies,
    ) {
        if let std::collections::btree_map::Entry::Vacant(e) =
            self.matched_writers.entry(remote_writer_guid)
        {
            if let Err(e) = self.qos.is_compatible(&qos) {
                self.reader_state_notifier
                    .send(DataReaderStatusChanged::RequestedIncompatibleQos(e.clone()))
                    .expect("couldn't send reader_state_notifier");
                warn!(
                "Reader requested incompatible qos from Writer\n\tWriter: {}\n\tReader: {}\n\terror: {}",
                self.guid, remote_writer_guid, e
            );
                return;
            }

            info!(
                "Reader found matched Writer\n\tReader: {}\n\tWriter: {}",
                self.guid, remote_writer_guid
            );

            e.insert(WriterProxy::new(
                remote_writer_guid,
                unicast_locator_list,
                multicast_locator_list,
                default_unicast_locator_list,
                default_multicast_locator_list,
                data_max_size_serialized,
                qos,
                self.reader_cache.clone(),
            ));
            let sub_match_state = SubscriptionMatchedStatus::new(
                (self.matched_writers.len() + self.unmatched_writers.len()) as i32,
                1,
                self.matched_writers.len() as i32,
                1,
                remote_writer_guid,
            );
            self.reader_state_notifier
                .send(DataReaderStatusChanged::SubscriptionMatched(
                    sub_match_state,
                ))
                .expect("couldn't send reader_state_notifier");
            self.reader_state_notifier
                .send(DataReaderStatusChanged::LivelinessChanged(
                    LivelinessChangedStatus::new(
                        self.matched_writers.len() as i32,
                        self.unmatched_writers.len() as i32,
                        1,
                        0,
                        remote_writer_guid,
                    ),
                ))
                .expect("couldn't send channel 'reader_state_notifier'");
        } else {
            let remote_writer = self.matched_writers.get_mut(&remote_writer_guid).unwrap();
            macro_rules! update_proxy_if_need {
                ($name:ident) => {
                    if remote_writer.$name != $name {
                        remote_writer.$name = $name;
                        info!(
                            "Reader update matched Writer info\n\tReader: {}\n\tWriter: {}",
                            self.guid, remote_writer_guid
                        );
                    }
                };
            }
            update_proxy_if_need!(qos);
            update_proxy_if_need!(unicast_locator_list);
            update_proxy_if_need!(multicast_locator_list);
            update_proxy_if_need!(default_unicast_locator_list);
            update_proxy_if_need!(default_multicast_locator_list);
            update_proxy_if_need!(data_max_size_serialized);
        }
    }

    pub fn is_writer_match(&self, topic_name: &str, data_type: &str) -> bool {
        self.topic.name() == topic_name && self.topic.type_desc() == data_type
    }
    /*
    pub fn matched_writer_lookup(&mut self, guid: GUID) -> Option<WriterProxy> {
        self.matched_writers
            .get_mut(&guid)
            .map(|proxy| proxy.clone())
    }
    */

    fn matched_writer_unmatch(&mut self, guid: GUID) {
        if let Some(writer_proxy) = self.matched_writers.remove(&guid) {
            self.unmatched_writers.insert(guid, writer_proxy);
            self.reader_state_notifier
                .send(DataReaderStatusChanged::LivelinessChanged(
                    LivelinessChangedStatus::new(
                        self.matched_writers.len() as i32,
                        self.unmatched_writers.len() as i32,
                        -1,
                        1,
                        guid,
                    ),
                ))
                .expect("couldn't send channel 'reader_state_notifier'");
        }
    }

    #[inline]
    fn send_sub_unmatch(&self, guid: GUID) {
        self.reader_state_notifier
            .send(DataReaderStatusChanged::SubscriptionMatched(
                SubscriptionMatchedStatus::new(
                    (self.matched_writers.len() + self.unmatched_writers.len()) as i32,
                    0,
                    self.matched_writers.len() as i32,
                    -1,
                    guid,
                ),
            ))
            .expect("couldn't send reader_state_notifier");
    }

    pub fn unmatched_writer_remove(&mut self, guid: GUID) {
        if self.unmatched_writers.remove(&guid).is_some() {
            self.send_sub_unmatch(guid);
        }
    }

    pub fn matched_writer_remove(&mut self, guid: GUID) {
        if self.matched_writers.remove(&guid).is_some() {
            self.reader_state_notifier
                .send(DataReaderStatusChanged::LivelinessChanged(
                    LivelinessChangedStatus::new(
                        self.matched_writers.len() as i32,
                        self.unmatched_writers.len() as i32,
                        -1,
                        1,
                        guid,
                    ),
                ))
                .expect("couldn't send channel 'reader_state_notifier'");
            self.send_sub_unmatch(guid);
        }
    }

    pub fn delete_writer_proxy(&mut self, guid_prefix: GuidPrefix) {
        let to_delete: Vec<GUID> = self
            .matched_writers
            .keys()
            .filter(|k| k.guid_prefix == guid_prefix)
            .copied()
            .collect();

        for d in to_delete {
            self.matched_writer_remove(d);
        }
        let to_delete: Vec<GUID> = self
            .unmatched_writers
            .keys()
            .filter(|k| k.guid_prefix == guid_prefix)
            .copied()
            .collect();

        for d in to_delete {
            self.unmatched_writer_remove(d);
        }
    }

    pub fn handle_gap(&mut self, writer_guid: GUID, gap: Gap) {
        if let Some(wp) = self.unmatched_writers.remove(&writer_guid) {
            self.matched_writers.insert(writer_guid, wp);
            self.reader_state_notifier
                .send(DataReaderStatusChanged::LivelinessChanged(
                    LivelinessChangedStatus::new(
                        self.matched_writers.len() as i32,
                        self.unmatched_writers.len() as i32,
                        1,
                        -1,
                        writer_guid,
                    ),
                ))
                .expect("couldn't send channel 'reader_state_notifier'");
        }
        if let Some(writer_proxy) = self.matched_writers.get_mut(&writer_guid) {
            let mut seq_num = gap.gap_start;
            while seq_num < gap.gap_list.base() {
                writer_proxy.irrelevant_change_set(seq_num);
                seq_num += SequenceNumber(1);
            }
            for seq_num in gap.gap_list.set() {
                writer_proxy.irrelevant_change_set(seq_num);
            }
        } else {
            warn!(
                "Reader tried handle GAP from unmatched Writer\n\tReader: {}\n\tWriter: {}",
                self.guid, writer_guid
            );
        }
    }

    pub fn handle_heartbeat(
        &mut self,
        writer_guid: GUID,
        hb_flag: BitFlags<HeartbeatFlag>,
        heartbeat: Heartbeat,
    ) {
        if let Some(wp) = self.unmatched_writers.remove(&writer_guid) {
            self.matched_writers.insert(writer_guid, wp);
            self.reader_state_notifier
                .send(DataReaderStatusChanged::LivelinessChanged(
                    LivelinessChangedStatus::new(
                        self.matched_writers.len() as i32,
                        self.unmatched_writers.len() as i32,
                        1,
                        -1,
                        writer_guid,
                    ),
                ))
                .expect("couldn't send channel 'reader_state_notifier'");
        }
        if let Some(writer_proxy) = self.matched_writers.get_mut(&writer_guid) {
            trace!(
                "Reader handle heartbeat {{ first_sn: {}, last_sn: {} }} from Writer\n\tReader: {}\n\tWriter: {}",
                heartbeat.first_sn.0,
                heartbeat.last_sn.0,
                self.guid,
                writer_guid,
            );

            writer_proxy.missing_changes_update(heartbeat.first_sn, heartbeat.last_sn);
            writer_proxy.lost_changes_update(heartbeat.first_sn);
        } else {
            warn!(
                "Reader tried handle Heartbeat from unmatched Writer\n\tReader: {}\n\tWriter: {}",
                self.guid, writer_guid
            );
            return;
        }
        if !hb_flag.contains(HeartbeatFlag::Final) {
            // to must_send_ack
            // Transition: T5
            // set timer whose duration is self.heartbeat_response_delay
            if self.heartbeat_response_delay == Duration::ZERO {
                trace!(
                    "Reader received Heartbeat: heartbeat_response_delay == 0\n\tReader: {}",
                    self.guid
                );
                self.handle_hb_response_timeout(writer_guid);
            } else {
                trace!(
                    "Reader send set_reader_hb_timer_sender to set Heartbeat timer\n\tReader: {}",
                    self.guid
                );
                self.set_reader_hb_timer_sender
                    .send((self.entity_id(), writer_guid))
                    .expect("couldn't send channel 'set_reader_hb_timer_sender'");
            }
        } else if !hb_flag.contains(HeartbeatFlag::Liveliness) {
            // to may_send_ack
            if let Some(writer_proxy) = self.matched_writers.get_mut(&writer_guid) {
                if writer_proxy.missing_changes().is_empty() {
                    // to waiting
                    // Transition: T3
                    // nothing to do
                } else {
                    // to must_send_ack
                    // Transition: T4

                    // Transition: T5
                    // set timer whose duration is self.heartbeat_response_delay
                    if self.heartbeat_response_delay == Duration::ZERO {
                        trace!(
                            "Reader received Heartbeat: heartbeat_response_delay == 0\n\tReader: {}",
                            self.guid
                        );
                        self.handle_hb_response_timeout(writer_guid);
                    } else {
                        trace!(
                            "Reader send set_reader_hb_timer_sender to set Heartbeat timer\n\tReader: {}",
                            self.guid
                        );
                        self.set_reader_hb_timer_sender
                            .send((self.entity_id(), writer_guid))
                            .expect("couldn't send channel 'set_reader_hb_timer_sender'");
                    }
                }
            }
        } else {
            // to waiting
            // nothing to do
        }
    }

    pub fn handle_hb_response_timeout(&mut self, writer_guid: GUID) {
        let self_guid_prefix = self.guid_prefix();
        let self_entity_id = self.entity_id();
        if let Some(writer_proxy) = self.matched_writers.get_mut(&writer_guid) {
            let mut missign_seq_num_set = Vec::new();
            for change in writer_proxy.missing_changes() {
                missign_seq_num_set.push(change);
            }
            // rtps 2.3 spec 8.4.12.2.4 say, "missing_seq_num_set.base := the_writer_proxy.available_changes_max() + 1;",
            // but this have problem.
            // for instance, when droped SeqNum(1) and received SeqNum(2), base is set to SeqNum(2)
            // and set is [SeqNum(1)]. Bitmap can't represent value which is smaller than base.
            // So, On RustDDS, when there is some missing SeqNum, base is set to the smallest
            // SeqNum on the set.
            let missign_seq_num_set_base = if missign_seq_num_set.is_empty() {
                writer_proxy.available_changes_max() + SequenceNumber(1)
            } else {
                *missign_seq_num_set.iter().min().unwrap()
            };
            let reader_sn_state =
                SequenceNumberSet::from_vec(missign_seq_num_set_base, missign_seq_num_set);
            let mut message_builder = MessageBuilder::new();
            message_builder.info_dst(self.endianness, writer_proxy.remote_writer_guid.guid_prefix);
            message_builder.acknack(
                self.endianness,
                writer_guid.entity_id,
                self_entity_id,
                reader_sn_state,
                1,
                false,
            );
            let message = message_builder.build(self_guid_prefix);
            let message_buf = message
                .write_to_vec_with_ctx(self.endianness)
                .expect("couldn't serialize message");
            let mut is_send = false;
            for uni_loc in writer_proxy.get_unicast_locator_list() {
                if uni_loc.kind == Locator::KIND_UDPV4 {
                    let port = uni_loc.port;
                    let addr = uni_loc.address;
                    info!(
                        "Reader send acknack(heartbeat response) message to {}.{}.{}.{}:{}\n\tReader: {}",
                        addr[12], addr[13], addr[14], addr[15], port,self.guid,
                    );
                    self.udp_sender.send_to_unicast(
                        &message_buf,
                        Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                        port as u16,
                    );
                    is_send = true;
                }
            }
            // if there is Participant on same host, umber_dds need to send acknack to multicast
            for mul_loc in writer_proxy.get_multicast_locator_list() {
                if mul_loc.kind == Locator::KIND_UDPV4 {
                    let port = mul_loc.port;
                    let addr = mul_loc.address;
                    info!(
                        "Reader send acknack(heartbeat response) message to {}.{}.{}.{}:{}\n\tReader: {}",
                        addr[12], addr[13], addr[14], addr[15], port,self.guid,
                    );
                    self.udp_sender.send_to_unicast(
                        &message_buf,
                        Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                        port as u16,
                    );
                    is_send = true;
                }
            }
            if !is_send {
                error!("attempt to send acknack, but not found UDP_V4 locator of Writer\n\tReader: {}\n\tWriter: {}", self.guid, writer_proxy.remote_writer_guid);
            }
        } else {
            warn!(
                "Reader tried send Heartbeat to Writer but, not found\n\tReader: {}\n\tWriter: {}",
                self.guid, writer_guid
            );
        }
    }

    pub fn heartbeat_response_delay(&self) -> StdDuration {
        StdDuration::new(
            self.heartbeat_response_delay.seconds as u64,
            self.heartbeat_response_delay.fraction,
        )
    }
    pub fn is_contain_writer(&self, writer_guid: GUID) -> bool {
        self.matched_writers.contains_key(&writer_guid)
            || self.unmatched_writers.contains_key(&writer_guid)
    }
    pub fn get_matched_writer_qos(&self, writer_guid: GUID) -> DataWriterQosPolicies {
        if let Some(wp) = self.matched_writers.get(&writer_guid) {
            wp.qos.clone()
        } else if let Some(wp) = self.unmatched_writers.get(&writer_guid) {
            wp.qos.clone()
        } else {
            panic!(
                "not found Writer matched to Reader\n\tReader: {}\n\tWriter: {}",
                self.guid, writer_guid,
            )
        }
    }

    pub fn check_liveliness(&mut self, disc_db: &mut DiscoveryDB) {
        let mut to_unmatch = Vec::new();
        for (guid, wp) in &self.matched_writers {
            let wld = wp.qos.liveliness().lease_duration;
            if wld == Duration::INFINITE {
                continue;
            }
            match disc_db.read_remote_writer(*guid) {
                EndpointState::Live(last_added) => {
                    let elapse =
                        Timestamp::now().expect("failed get Timestamp::now()") - last_added;
                    if elapse > wld {
                        debug!("checked liveliness of writer Lost, ld: {:?}, elapse: {:?}\n\tReader: {}\n\tWriter: {}", wld, elapse, self.guid, guid);
                        to_unmatch.push(*guid);
                    }
                    debug!("checked liveliness of writer, ld: {:?}, elapse: {:?}\n\tReader: {}\n\tWriter: {}", wld, elapse, self.guid, guid);
                }
                EndpointState::LivelinessLost => to_unmatch.push(*guid),
                EndpointState::Unknown => debug!("reader requested check liveliness of Unknown Writer\n\tReader: {}\n\tWriter: {}", self.guid, guid),
            }
        }
        for g in to_unmatch {
            disc_db.update_remote_writer_state(g, EndpointState::LivelinessLost);
            self.matched_writer_unmatch(g);
        }
    }

    pub fn get_min_remote_writer_lease_duration(&self) -> StdDuration {
        let mut min_ld = Duration::INFINITE;
        for wp in self.matched_writers.values() {
            let wld = wp.qos.liveliness().lease_duration;
            if wld < min_ld {
                min_ld = wld;
            }
        }
        if min_ld == Duration::INFINITE {
            StdDuration::new(10, 0)
        } else {
            StdDuration::new(min_ld.seconds as u64, min_ld.fraction)
        }
    }
}

/// For more details on each variants, please refer to the DDS specification. DDS v1.4 spec, 2.2.4 Listeners, Conditions, and Wait-sets (<https://www.omg.org/spec/DDS/1.4/PDF#G5.1034386>)
///
/// The content for each variant has not been implemented yet, but it is planned to be implemented in the future.
pub enum DataReaderStatusChanged {
    SampleRejected,
    LivelinessChanged(LivelinessChangedStatus),
    RequestedDeadlineMissed,
    RequestedIncompatibleQos(String),
    DataAvailable,
    SampleLost,
    SubscriptionMatched(SubscriptionMatchedStatus),
}

pub struct SubscriptionMatchedStatus {
    pub total_count: i32,
    pub total_count_change: i32,
    pub current_count: i32,
    pub current_count_change: i32,
    /// This is diffarent form DDS spec.
    /// The GUID is remote writer's one.
    pub guid: GUID,
}

impl SubscriptionMatchedStatus {
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

pub struct LivelinessChangedStatus {
    pub alive_count: i32,
    pub not_alive_count: i32,
    pub alive_count_change: i32,
    pub not_alive_count_change: i32,
    /// This is diffarent form DDS spec.
    /// The GUID is remote writer's one.
    pub guid: GUID,
}

impl LivelinessChangedStatus {
    pub fn new(
        alive_count: i32,
        not_alive_count: i32,
        alive_count_change: i32,
        not_alive_count_change: i32,
        guid: GUID,
    ) -> Self {
        Self {
            alive_count,
            not_alive_count,
            alive_count_change,
            not_alive_count_change,
            guid,
        }
    }
}

pub struct ReaderIngredients {
    // Entity
    pub guid: GUID,
    // Endpoint
    pub reliability_level: ReliabilityQosKind,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    // Reader
    pub expectsinline_qos: bool,
    pub heartbeat_response_delay: Duration,
    pub rhc: Arc<RwLock<HistoryCache>>,
    // This implementation spesific
    pub topic: Topic,
    pub qos: DataReaderQosPolicies,
    pub reader_state_notifier: mio_channel::Sender<DataReaderStatusChanged>,
}

impl RTPSEntity for Reader {
    fn guid(&self) -> GUID {
        self.guid
    }
}
