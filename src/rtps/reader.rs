use crate::dds::{
    qos::{policy::ReliabilityQosKind, DataReaderQosPolicies, DataWriterQosPolicies},
    Topic,
};
use crate::discovery::structure::data::DiscoveredReaderData;
use crate::message::message_builder::MessageBuilder;
use crate::message::submessage::{
    element::{Gap, Heartbeat, Locator, SequenceNumber, SequenceNumberSet},
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
use colored::*;
use core::net::Ipv4Addr;
use core::time::Duration as StdDuration;
use enumflags2::BitFlags;
use mio_extras::channel as mio_channel;
use speedy::{Endianness, Writable};
use std::sync::RwLock;

/// RTPS StatefulReader
pub struct Reader {
    // Entity
    guid: GUID,
    // Endpoint
    _topic_kind: TopicKind,
    reliability_level: ReliabilityQosKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    // Reader
    expectsinline_qos: bool,
    heartbeat_response_delay: Duration,
    reader_cache: Arc<RwLock<HistoryCache>>,
    // StatefulReader
    matched_writers: BTreeMap<GUID, WriterProxy>,
    // This implementation spesific
    topic: Topic,
    qos: DataReaderQosPolicies,
    endianness: Endianness,
    reader_state_notifier: mio_channel::Sender<DataReaderStatusChanged>,
    set_reader_hb_timer_sender: mio_channel::Sender<(EntityId, GUID)>,
    sender: Rc<UdpSender>,
}

impl Reader {
    pub fn new(
        ri: ReaderIngredients,
        sender: Rc<UdpSender>,
        set_reader_hb_timer_sender: mio_channel::Sender<(EntityId, GUID)>,
    ) -> Self {
        Self {
            guid: ri.guid,
            _topic_kind: ri.topic_kind,
            reliability_level: ri.reliability_level,
            unicast_locator_list: ri.unicast_locator_list,
            multicast_locator_list: ri.multicast_locator_list,
            expectsinline_qos: ri.expectsinline_qos,
            heartbeat_response_delay: ri.heartbeat_response_delay,
            reader_cache: ri.rhc,
            matched_writers: BTreeMap::new(),
            topic: ri.topic,
            qos: ri.qos,
            endianness: Endianness::LittleEndian,
            reader_state_notifier: ri.reader_state_notifier,
            set_reader_hb_timer_sender,
            sender,
        }
    }

    pub fn is_reliable(&self) -> bool {
        match self.reliability_level {
            ReliabilityQosKind::Reliable => true,
            ReliabilityQosKind::BestEffort => false,
        }
    }

    pub fn sedp_data(&self) -> DiscoveredReaderData {
        let proxy = ReaderProxy::new(
            self.guid,
            self.expectsinline_qos,
            self.unicast_locator_list.clone(),
            self.multicast_locator_list.clone(),
            self.qos.clone(),
            Arc::new(RwLock::new(HistoryCache::new())),
        );
        let sub_data = self.topic.sub_builtin_topic_data();
        DiscoveredReaderData::new(proxy, sub_data)
    }

    fn print_self_info(&self) {
        if cfg!(debug_assertions) {
            eprintln!("<{}>: Reader info", "Reader: Info".green());
            eprintln!("\tguid: {:?}", self.guid);
            eprintln!("\tunicast locators");
            for loc in &self.unicast_locator_list {
                eprintln!("\t\t{:?}", loc)
            }
            eprintln!("\tmulticast locators");
            for loc in &self.multicast_locator_list {
                eprintln!("\t\t{:?}", loc)
            }
            eprintln!("\tmatched writers");
            for (eid, reader) in self.matched_writers.iter() {
                eprintln!("\t\treader guid: {:?}", eid);
                eprintln!("\tunicast locators");
                for loc in &reader.unicast_locator_list {
                    eprintln!("\t\t{:?}", loc)
                }
                eprintln!("\tmulticast locators");
                for loc in &reader.multicast_locator_list {
                    eprintln!("\t\t{:?}", loc)
                }
            }
        }
    }

    pub fn add_change(&mut self, source_guid_prefix: GuidPrefix, change: CacheChange) {
        let writer_guid = GUID::new(source_guid_prefix, change.writer_guid.entity_id);
        if self.is_reliable() {
            // Reliable Reader Behavior
            if self
                .reader_cache
                .write()
                .expect("couldn't write reader_cache")
                .add_change(change.clone())
                .is_err()
            {
                return;
            }
            eprintln!(
                "<{}>: reliable reader, add change to reader_cache",
                "Reader: Info".green()
            );
            self.reader_state_notifier
                .send(DataReaderStatusChanged::DataAvailable)
                .expect("couldn't send channel 'reader_state_notifier'");
            if let Some(writer_proxy) = self.matched_writers.get_mut(&writer_guid) {
                writer_proxy.received_chage_set(change.sequence_number);
            }
        } else {
            // BestEffort Reader Behavior
            if self.matched_writer_lookup(writer_guid).is_some() {
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
                    if self
                        .reader_cache
                        .write()
                        .expect("couldn't write reader_cache")
                        .add_change(change.clone())
                        .is_err()
                    {
                        return;
                    }
                    eprintln!(
                        "<{}>: besteffort reader, add change to reader_cache",
                        "Reader: Info".green()
                    );
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
                    eprintln!("<{}>: dosen't write change to reader_cache, because change.sequence_number < expected_seq_num. This Reader is BestEffort, so resned won't execute.", "Reader: Warn".yellow());
                }
            } else {
                self.print_self_info();
                eprintln!(
                    "<{}>: couldn't find writer_proxy which has guid {:?}",
                    "Reader: Warn".yellow(),
                    writer_guid
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
        eprintln!(
            "<{}>: add matched Writer which has {:?}",
            "Reader: Info".green(),
            remote_writer_guid
        );
        if let Err(e) = self.qos.is_compatible(&qos) {
            self.reader_state_notifier
                .send(DataReaderStatusChanged::RequestedIncompatibleQos)
                .expect("couldn't send reader_state_notifier");
            eprintln!(
                "<{}>: add matched Writer which has {:?} failed. {}",
                "Reader: Warn".yellow(),
                remote_writer_guid,
                e
            );
            return;
        }

        self.reader_state_notifier
            .send(DataReaderStatusChanged::SubscriptionMatched(
                remote_writer_guid,
            ))
            .expect("couldn't send reader_state_notifier");
        self.matched_writers.insert(
            remote_writer_guid,
            WriterProxy::new(
                remote_writer_guid,
                unicast_locator_list,
                multicast_locator_list,
                data_max_size_serialized,
                qos,
                self.reader_cache.clone(),
            ),
        );
    }
    pub fn is_writer_match(&self, topic_name: &str, data_type: &str) -> bool {
        self.topic.name() == topic_name && self.topic.type_desc() == data_type
    }
    pub fn matched_writer_lookup(&mut self, guid: GUID) -> Option<WriterProxy> {
        self.matched_writers
            .get_mut(&guid)
            .map(|proxy| proxy.clone())
    }
    #[allow(dead_code)]
    pub fn matched_writer_remove(&mut self, guid: GUID) {
        self.matched_writers.remove(&guid);
    }

    pub fn handle_gap(&mut self, writer_guid: GUID, gap: Gap) {
        if let Some(writer_proxy) = self.matched_writers.get_mut(&writer_guid) {
            let mut seq_num = gap.gap_start;
            while seq_num < gap.gap_list.base() {
                writer_proxy.irrelevant_change_set(seq_num);
                seq_num += SequenceNumber(1);
            }
            for seq_num in gap.gap_list.set() {
                writer_proxy.irrelevant_change_set(seq_num);
            }
        }
    }

    pub fn handle_heartbeat(
        &mut self,
        writer_guid: GUID,
        hb_flag: BitFlags<HeartbeatFlag>,
        heartbeat: Heartbeat,
    ) {
        if let Some(writer_proxy) = self.matched_writers.get_mut(&writer_guid) {
            eprintln!(
                "<{}>: handle heartbeat {{ first_sn: {}, last_sn: {} }} from writer which has {:?}",
                "Reader: Info".green(),
                heartbeat.first_sn.0,
                heartbeat.last_sn.0,
                writer_guid,
            );

            writer_proxy.missing_changes_update(heartbeat.first_sn, heartbeat.last_sn);
            writer_proxy.lost_changes_update(heartbeat.first_sn);
        } else {
            eprintln!(
                "<{}>: couldn't find reader which has {:?}",
                "Reader: Warn".yellow(),
                writer_guid
            );
            return;
        }
        if !hb_flag.contains(HeartbeatFlag::Final) {
            // to must_send_ack
            // Transition: T5
            // set timer whose duration is self.heartbeat_response_delay
            if self.heartbeat_response_delay == Duration::ZERO {
                eprintln!(
                    "<{}>: heartbeat_response_delay == 0",
                    "Reader: Info".green(),
                );
                self.handle_hb_response_timeout(writer_guid);
            } else {
                eprintln!(
                    "<{}>: set_reader_hb_timer_sender sned",
                    "Reader: Info".green(),
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
                        eprintln!(
                            "<{}>: heartbeat_response_delay == 0",
                            "Reader: Info".green(),
                        );
                        self.handle_hb_response_timeout(writer_guid);
                    } else {
                        eprintln!(
                            "<{}>: set_reader_hb_timer_sender sned",
                            "Reader: Info".green(),
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
            for uni_loc in &writer_proxy.unicast_locator_list {
                if uni_loc.kind == Locator::KIND_UDPV4 {
                    let port = uni_loc.port;
                    let addr = uni_loc.address;
                    eprintln!(
                        "<{}>: sned acknack(heartbeat response) message to {}.{}.{}.{}:{}",
                        "Reader: Info".green(),
                        addr[12],
                        addr[13],
                        addr[14],
                        addr[15],
                        port
                    );
                    self.sender.send_to_unicast(
                        &message_buf,
                        Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                        port as u16,
                    );
                }
            }
            // if there is Participant on same host, umber_dds need to send acknack to multicast
            for mul_loc in &writer_proxy.multicast_locator_list {
                if mul_loc.kind == Locator::KIND_UDPV4 {
                    let port = mul_loc.port;
                    let addr = mul_loc.address;
                    eprintln!(
                        "<{}>: sned acknack(heartbeat response) message to {}.{}.{}.{}:{}",
                        "Reader: Info".green(),
                        addr[12],
                        addr[13],
                        addr[14],
                        addr[15],
                        port
                    );
                    self.sender.send_to_unicast(
                        &message_buf,
                        Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                        port as u16,
                    );
                }
            }
        } else {
            eprintln!(
                "<{}>: couldn't find reader which has {:?}",
                "Reader: Warn".yellow(),
                writer_guid
            );
        }
    }

    pub fn heartbeat_response_delay(&self) -> StdDuration {
        StdDuration::new(
            self.heartbeat_response_delay.seconds as u64,
            self.heartbeat_response_delay.fraction,
        )
    }
    pub fn is_contain_writer(&self, writer_entity_id: EntityId) -> bool {
        for guid in self.matched_writers.keys() {
            if guid.entity_id == writer_entity_id {
                return true;
            }
        }
        false
    }
}

/// For more details on each variants, please refer to the DDS specification. DDS v1.4 spec, 2.2.4 Listeners, Conditions, and Wait-sets (<https://www.omg.org/spec/DDS/1.4/PDF#G5.1034386>)
///
/// The content for each variant has not been implemented yet, but it is planned to be implemented in the future.
pub enum DataReaderStatusChanged {
    SampleRejected,
    LivelinessChanged,
    RequestedDeadlineMissed,
    RequestedIncompatibleQos,
    DataAvailable,
    SampleLost,
    /// This is diffarent form DDS spec.
    /// The GUID is remote writer's one.
    SubscriptionMatched(GUID),
}

pub struct ReaderIngredients {
    // Entity
    pub guid: GUID,
    // Endpoint
    pub topic_kind: TopicKind,
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
