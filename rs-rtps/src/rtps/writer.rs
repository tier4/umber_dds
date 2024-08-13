use crate::dds::{qos::policy::ReliabilityQosKind, topic::Topic};
use crate::discovery::structure::data::DiscoveredWriterData;
use crate::message::{
    message_builder::MessageBuilder,
    submessage::element::{
        acknack::AckNack, Count, Locator, SequenceNumber, SequenceNumberSet, SerializedPayload,
        Timestamp,
    },
};
use crate::network::udp_sender::UdpSender;
use crate::rtps::cache::{
    CacheChange, ChangeForReaderStatusKind, ChangeKind, HistoryCache, InstantHandle,
};
use crate::rtps::reader_locator::ReaderLocator;
use crate::structure::{
    duration::Duration,
    entity::RTPSEntity,
    entity_id::EntityId,
    guid::GUID,
    proxy::{ReaderProxy, WriterProxy},
    topic_kind::TopicKind,
};
use colored::*;
use mio_extras::channel as mio_channel;
use mio_v06::Token;
use speedy::{Endianness, Writable};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::Duration as StdDuration;

/// RTPS StatefulWriter
pub struct Writer {
    // Entity
    guid: GUID,
    // Endpoint
    _topic_kind: TopicKind,
    reliability_level: ReliabilityQosKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    // Writer
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    _nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    writer_cache: Arc<RwLock<HistoryCache>>,
    data_max_size_serialized: i32,
    // StatelessWriter
    reader_locators: Vec<ReaderLocator>,
    // StatefulWriter
    matched_readers: HashMap<GUID, ReaderProxy>,
    // This implementation spesific
    topic: Topic,
    endianness: Endianness,
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
    set_writer_nack_sender: mio_channel::Sender<(EntityId, GUID)>,
    sender: Rc<UdpSender>,
    hb_counter: Count,
    an_state: AckNackState,
}

#[derive(PartialEq, Eq)]
enum AckNackState {
    Waiting,
    Repairing,
    MutsRepair,
}

impl Writer {
    pub fn new(
        wi: WriterIngredients,
        sender: Rc<UdpSender>,
        set_writer_nack_sender: mio_channel::Sender<(EntityId, GUID)>,
    ) -> Self {
        Self {
            guid: wi.guid,
            _topic_kind: wi.topic_kind,
            reliability_level: wi.reliability_level,
            unicast_locator_list: wi.unicast_locator_list,
            multicast_locator_list: wi.multicast_locator_list,
            push_mode: wi.push_mode,
            heartbeat_period: wi.heartbeat_period,
            nack_response_delay: wi.nack_response_delay,
            _nack_suppression_duration: wi.nack_suppression_duration,
            last_change_sequence_number: SequenceNumber(0),
            writer_cache: Arc::new(RwLock::new(HistoryCache::new())),
            data_max_size_serialized: wi.data_max_size_serialized,
            reader_locators: Vec::new(),
            matched_readers: HashMap::new(),
            topic: wi.topic,
            endianness: Endianness::LittleEndian,
            writer_command_receiver: wi.writer_command_receiver,
            set_writer_nack_sender,
            sender,
            hb_counter: 0,
            an_state: AckNackState::Waiting,
        }
    }

    pub fn sedp_data(&self) -> DiscoveredWriterData {
        let proxy = WriterProxy::new(
            self.guid,
            self.unicast_locator_list.clone(),
            self.multicast_locator_list.clone(),
            self.data_max_size_serialized,
            Arc::new(RwLock::new(HistoryCache::new())),
        );
        let pub_data = self.topic.pub_builtin_topic_data();
        DiscoveredWriterData::new(proxy, pub_data)
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<SerializedPayload>,
        handle: InstantHandle,
    ) -> CacheChange {
        // Writer
        self.last_change_sequence_number += SequenceNumber(1);
        CacheChange::new(
            kind,
            self.guid,
            self.last_change_sequence_number,
            data,
            handle,
        )
    }

    pub fn is_reliable(&self) -> bool {
        match self.reliability_level {
            ReliabilityQosKind::Reliable => true,
            ReliabilityQosKind::BestEffort => false,
        }
    }

    pub fn reader_locator_add(&mut self, locator: ReaderLocator) {
        // StatelessWriter
        self.reader_locators.push(locator)
    }

    pub fn reader_locator_remove(&mut self, locator: ReaderLocator) {
        // StatelessWriter
        if let Some(loc_pos) = self.reader_locators.iter().position(|rl| *rl == locator) {
            self.reader_locators.remove(loc_pos);
        }
    }

    pub fn unsent_changes_reset(&mut self) {
        // StatelessWriter
        for rl in &mut self.reader_locators {
            rl.unsent_changes = self
                .writer_cache
                .read()
                .expect("couldn't read writer_cache")
                .changes
                .values()
                .cloned()
                .collect();
        }
    }

    pub fn entity_token(&self) -> Token {
        self.entity_id().as_token()
    }

    fn print_self_info(&self) {
        if cfg!(debug_assertions) {
            eprintln!("<{}>: Writer info", "Writer: Info".green(),);
            eprintln!("\tguid: {:?}", self.guid);
            eprintln!("\tunicast locators");
            for loc in &self.unicast_locator_list {
                eprintln!("\t\t{:?}", loc)
            }
            eprintln!("\tmulticast locators");
            for loc in &self.multicast_locator_list {
                eprintln!("\t\t{:?}", loc)
            }
            eprintln!("\tmatched readers");
            for (eid, reader) in self.matched_readers.iter() {
                eprintln!("\t\treader guid: {:?}", eid);
                eprintln!("\t\tunicast locators");
                for loc in &reader.unicast_locator_list {
                    eprintln!("\t\t\t{:?}", loc)
                }
                eprintln!("\t\tmulticast locators");
                for loc in &reader.multicast_locator_list {
                    eprintln!("\t\t\t{:?}", loc)
                }
            }
        }
    }

    pub fn handle_writer_cmd(&mut self) {
        while let Ok(cmd) = self.writer_command_receiver.try_recv() {
            // this is new_change
            self.last_change_sequence_number += SequenceNumber(1);
            let a_change = CacheChange::new(
                ChangeKind::Alive,
                self.guid,
                self.last_change_sequence_number,
                cmd.serialized_payload,
                InstantHandle {},
            );
            // register a_change to writer HistoryCache
            self.add_change_to_hc(a_change.clone());
            let self_guid_prefix = self.guid_prefix();
            self.print_self_info();
            for (_guid, reader_proxy) in &mut self.matched_readers {
                while let Some(change_for_reader) = reader_proxy.next_unsent_change() {
                    reader_proxy.update_cache_state(
                        change_for_reader.seq_num,
                        change_for_reader.is_relevant,
                        ChangeForReaderStatusKind::Underway,
                    );
                    if change_for_reader.is_relevant {
                        if let Some(aa_change) = self
                            .writer_cache
                            .read()
                            .expect("couldn't read writer_cache")
                            .get_change(change_for_reader.seq_num)
                        {
                            // build RTPS Message
                            let mut message_builder = MessageBuilder::new();
                            let time_stamp = Timestamp::now();
                            message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                            message_builder.data(
                                Endianness::LittleEndian,
                                reader_proxy.remote_reader_guid.entity_id,
                                self.guid.entity_id,
                                aa_change,
                            );
                            let message = message_builder.build(self_guid_prefix);
                            let message_buf = message
                                .write_to_vec_with_ctx(self.endianness)
                                .expect("couldn't serialize message");

                            // TODO:
                            // unicastとmulticastの両方に送信する必要はないから、状況によって切り替えるようにする。
                            for uni_loc in &reader_proxy.unicast_locator_list {
                                if uni_loc.kind == Locator::KIND_UDPV4 {
                                    let port = uni_loc.port;
                                    let addr = uni_loc.address;
                                    eprintln!(
                                        "<{}>: send data message to {}.{}.{}.{}:{}",
                                        "Writer: Info".green(),
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
                            for mul_loc in &reader_proxy.multicast_locator_list {
                                if mul_loc.kind == Locator::KIND_UDPV4 {
                                    let port = mul_loc.port;
                                    let addr = mul_loc.address;
                                    eprintln!(
                                        "<{}>: send data message to {}.{}.{}.{}:{}",
                                        "Writer: Info".green(),
                                        addr[12],
                                        addr[13],
                                        addr[14],
                                        addr[15],
                                        port
                                    );
                                    self.sender.send_to_multicast(
                                        &message_buf,
                                        Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                        port as u16,
                                    );
                                }
                            }
                        } else {
                            unreachable!();
                        }
                    } else {
                        // build RTPS Message
                        let mut message_builder = MessageBuilder::new();
                        // let time_stamp = Timestamp::now();
                        // message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                        let gap_start = change_for_reader.seq_num;
                        message_builder.gap(
                            Endianness::LittleEndian,
                            self.guid.entity_id,
                            reader_proxy.remote_reader_guid.entity_id,
                            gap_start,
                            SequenceNumberSet::from_vec(gap_start + SequenceNumber(1), vec![]),
                        );
                        let message = message_builder.build(self_guid_prefix);
                        let message_buf = message
                            .write_to_vec_with_ctx(self.endianness)
                            .expect("couldn't serialize message");
                        // TODO:
                        // unicastとmulticastの両方に送信する必要はないから、状況によって切り替えるようにする。
                        for uni_loc in &reader_proxy.unicast_locator_list {
                            if uni_loc.kind == Locator::KIND_UDPV4 {
                                let port = uni_loc.port;
                                let addr = uni_loc.address;
                                eprintln!(
                                    "<{}>: send data message to {}.{}.{}.{}:{}",
                                    "Writer: Info".green(),
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
                        for mul_loc in &reader_proxy.multicast_locator_list {
                            if mul_loc.kind == Locator::KIND_UDPV4 {
                                let port = mul_loc.port;
                                let addr = mul_loc.address;
                                eprintln!(
                                    "<{}>: send data message to {}.{}.{}.{}:{}",
                                    "Writer: Info".green(),
                                    addr[12],
                                    addr[13],
                                    addr[14],
                                    addr[15],
                                    port
                                );
                                self.sender.send_to_multicast(
                                    &message_buf,
                                    Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                    port as u16,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn send_heart_beat(&mut self) {
        let time_stamp = Timestamp::now();
        let writer_cache = self
            .writer_cache
            .read()
            .expect("couldn't read writer_cache");
        self.hb_counter += 1;
        let self_guid_prefix = self.guid_prefix();
        let self_entity_id = self.entity_id();
        self.print_self_info();
        for (_guid, reader_proxy) in &mut self.matched_readers {
            let mut message_builder = MessageBuilder::new();
            message_builder.info_ts(Endianness::LittleEndian, time_stamp);
            message_builder.heartbeat(
                self.endianness,
                self_entity_id,
                reader_proxy.remote_reader_guid.entity_id,
                writer_cache.get_seq_num_min(),
                writer_cache.get_seq_num_max(),
                self.hb_counter - 1,
                false,
            );
            let msg = message_builder.build(self_guid_prefix);
            let message_buf = msg
                .write_to_vec_with_ctx(self.endianness)
                .expect("couldn't serialize message");
            for uni_loc in &reader_proxy.unicast_locator_list {
                if uni_loc.kind == Locator::KIND_UDPV4 {
                    let port = uni_loc.port;
                    let addr = uni_loc.address;
                    eprintln!(
                        "<{}>: send heartbeat message to {}.{}.{}.{}:{}",
                        "Writer: Info".green(),
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
            for mul_loc in &reader_proxy.multicast_locator_list {
                if mul_loc.kind == Locator::KIND_UDPV4 {
                    let port = mul_loc.port;
                    let addr = mul_loc.address;
                    eprintln!(
                        "<{}>: send heartbeat message to {}.{}.{}.{}:{}",
                        "Writer: Info".green(),
                        addr[12],
                        addr[13],
                        addr[14],
                        addr[15],
                        port
                    );
                    self.sender.send_to_multicast(
                        &message_buf,
                        Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                        port as u16,
                    );
                }
            }
        }
    }

    fn add_change_to_hc(&mut self, change: CacheChange) {
        // add change to WriterHistoryCache & set status to Unset on each ReaderProxy
        self.writer_cache
            .write()
            .expect("couldn't write writer_cache")
            .add_change(change);
        for (_guid, reader_proxy) in &mut self.matched_readers {
            reader_proxy.update_cache_state(
                self.last_change_sequence_number,
                /* TODO: if DDS_FILTER(reader_proxy, change) { false } else { true }, */
                true,
                if self.push_mode {
                    ChangeForReaderStatusKind::Unsent
                } else {
                    ChangeForReaderStatusKind::Unacknowledged
                },
            )
        }
    }

    pub fn handle_acknack(&mut self, acknack: AckNack, reader_guid: GUID) {
        if let Some(reader_proxy) = self.matched_readers.get_mut(&reader_guid) {
            eprintln!(
                "<{}>: handle acknack from reader which has guid {:?}",
                "Writer: Info".green(),
                reader_guid
            );
            reader_proxy.acked_changes_set(acknack.reader_sn_state.base() - SequenceNumber(1));
            reader_proxy.requested_changes_set(acknack.reader_sn_state.set());
            reader_proxy.print_cache_states();
            match self.an_state {
                AckNackState::Waiting => {
                    // Transistion T9
                    if reader_proxy.requested_changes().len() != 0 {
                        self.an_state = AckNackState::MutsRepair
                    }
                }
                AckNackState::MutsRepair => {
                    // Transistion T10
                    if self.nack_response_delay == Duration::ZERO {
                        self.handle_nack_response_timeout(reader_guid);
                    } else {
                        self.set_writer_nack_sender
                            .send((self.entity_id(), reader_guid))
                            .expect("couldn't send channel 'set_writer_nack_sender'");
                    }
                }
                AckNackState::Repairing => unreachable!(),
            }
        } else {
            eprintln!(
                "<{}>: couldn't find reader_proxy which has guid {:?}",
                "Writer: Warn".yellow(),
                reader_guid
            );
        }
    }

    pub fn handle_nack_response_timeout(&mut self, reader_guid: GUID) {
        self.an_state = AckNackState::Repairing;
        let self_guid_prefix = self.guid_prefix();
        if let Some(reader_proxy) = self.matched_readers.get_mut(&reader_guid) {
            while let Some(change) = reader_proxy.next_requested_change() {
                reader_proxy.update_cache_state(
                    change.seq_num,
                    change.is_relevant,
                    ChangeForReaderStatusKind::Underway,
                );
                if change.is_relevant {
                    if let Some(aa_change) = self
                        .writer_cache
                        .read()
                        .expect("couldn't read writer_cache")
                        .get_change(change.seq_num)
                    {
                        // build RTPS Message
                        let mut message_builder = MessageBuilder::new();
                        let time_stamp = Timestamp::now();
                        message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                        message_builder.data(
                            Endianness::LittleEndian,
                            reader_proxy.remote_reader_guid.entity_id,
                            self.guid.entity_id,
                            aa_change,
                        );
                        let message = message_builder.build(self_guid_prefix);
                        let message_buf = message
                            .write_to_vec_with_ctx(self.endianness)
                            .expect("couldn't serialize message");

                        // TODO:
                        // unicastとmulticastの両方に送信する必要はないから、状況によって切り替えるようにする。
                        for uni_loc in &reader_proxy.unicast_locator_list {
                            if uni_loc.kind == Locator::KIND_UDPV4 {
                                let port = uni_loc.port;
                                let addr = uni_loc.address;
                                eprintln!(
                                    "<{}>: resend data message to {}.{}.{}.{}:{}",
                                    "Writer: Info".green(),
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
                        for mul_loc in &reader_proxy.multicast_locator_list {
                            if mul_loc.kind == Locator::KIND_UDPV4 {
                                let port = mul_loc.port;
                                let addr = mul_loc.address;
                                eprintln!(
                                    "<{}>: resend data message to {}.{}.{}.{}:{}",
                                    "Writer: Info".green(),
                                    addr[12],
                                    addr[13],
                                    addr[14],
                                    addr[15],
                                    port
                                );
                                self.sender.send_to_multicast(
                                    &message_buf,
                                    Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                    port as u16,
                                );
                            }
                        }
                    } else {
                        todo!(); // TODO: send Heartbeat
                                 // rtps spec, 8.4.2.2.4 Writers must eventually respond to a negative acknowledgment (reliable only)
                    }
                } else {
                    // build RTPS Message
                    let mut message_builder = MessageBuilder::new();
                    // let time_stamp = Timestamp::now();
                    // message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                    let gap_start = change.seq_num;
                    message_builder.gap(
                        Endianness::LittleEndian,
                        self.guid.entity_id,
                        reader_proxy.remote_reader_guid.entity_id,
                        gap_start,
                        SequenceNumberSet::from_vec(gap_start + SequenceNumber(1), vec![]),
                    );
                    let message = message_builder.build(self_guid_prefix);
                    let message_buf = message
                        .write_to_vec_with_ctx(self.endianness)
                        .expect("couldn't serialize message");
                    // TODO:
                    // unicastとmulticastの両方に送信する必要はないから、状況によって切り替えるようにする。
                    for uni_loc in &reader_proxy.unicast_locator_list {
                        if uni_loc.kind == Locator::KIND_UDPV4 {
                            let port = uni_loc.port;
                            let addr = uni_loc.address;
                            eprintln!(
                                "<{}>: send gap message to {}.{}.{}.{}:{}",
                                "Writer: Info".green(),
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
                    for mul_loc in &reader_proxy.multicast_locator_list {
                        if mul_loc.kind == Locator::KIND_UDPV4 {
                            let port = mul_loc.port;
                            let addr = mul_loc.address;
                            eprintln!(
                                "<{}>: send gap data message to {}.{}.{}.{}:{}",
                                "Writer: Info".green(),
                                addr[12],
                                addr[13],
                                addr[14],
                                addr[15],
                                port
                            );
                            self.sender.send_to_multicast(
                                &message_buf,
                                Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                port as u16,
                            );
                        }
                    }
                };
            }
        }
        self.an_state = AckNackState::Waiting;
    }

    pub fn matched_reader_add(
        &mut self,
        remote_reader_guid: GUID,
        expects_inline_qos: bool,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
    ) {
        eprintln!(
            "<{}>: add matched Reader which has {:?}",
            "Writer: Info".green(),
            remote_reader_guid
        );
        self.matched_readers.insert(
            remote_reader_guid,
            ReaderProxy::new(
                remote_reader_guid,
                expects_inline_qos,
                unicast_locator_list,
                multicast_locator_list,
                self.writer_cache.clone(),
            ),
        );
    }
    pub fn is_reader_match(&self, topic_name: &str, data_type: &str) -> bool {
        self.topic.name() == topic_name && self.topic.type_desc() == data_type
    }
    pub fn matched_reader_loolup(&self, guid: GUID) -> Option<ReaderProxy> {
        match self.matched_readers.get(&guid) {
            Some(prxy) => Some(prxy.clone()),
            None => None,
        }
    }
    pub fn matched_reader_remove(&mut self, guid: GUID) {
        self.matched_readers.remove(&guid);
    }

    pub fn heartbeat_period(&self) -> StdDuration {
        StdDuration::new(
            self.heartbeat_period.seconds as u64,
            self.heartbeat_period.fraction,
        )
    }

    pub fn nack_response_delay(&self) -> StdDuration {
        StdDuration::new(
            self.nack_response_delay.seconds as u64,
            self.nack_response_delay.fraction,
        )
    }
}

impl RTPSEntity for Writer {
    fn guid(&self) -> GUID {
        self.guid
    }
}

pub struct WriterIngredients {
    // Entity
    pub guid: GUID,
    // Endpoint
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityQosKind,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    // Writer
    pub push_mode: bool,
    pub heartbeat_period: Duration,
    pub nack_response_delay: Duration,
    pub nack_suppression_duration: Duration,
    pub data_max_size_serialized: i32,
    // This implementation spesific
    pub topic: Topic,
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
}
pub struct WriterCmd {
    pub serialized_payload: Option<SerializedPayload>,
}
