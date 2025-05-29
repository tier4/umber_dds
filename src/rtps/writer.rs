use crate::dds::{
    qos::{policy::ReliabilityQosKind, DataReaderQosPolicies, DataWriterQosPolicies},
    Topic,
};
use crate::discovery::structure::data::DiscoveredWriterData;
use crate::message::{
    message_builder::MessageBuilder,
    submessage::element::{
        AckNack, Count, Locator, SequenceNumber, SequenceNumberSet, SerializedPayload, Timestamp,
    },
};
use crate::network::udp_sender::UdpSender;
use crate::rtps::cache::{
    CacheChange, ChangeForReaderStatusKind, ChangeKind, HistoryCache, InstantHandle,
};
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
    last_change_sequence_number: SequenceNumber,
    writer_cache: Arc<RwLock<HistoryCache>>,
    data_max_size_serialized: i32,
    // StatelessWriter
    reader_locators: Vec<ReaderLocator>,
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
    sender: Rc<UdpSender>,
    hb_counter: Count,
    an_state: AckNackState,
}

#[derive(PartialEq, Eq)]
enum AckNackState {
    Waiting,
    Repairing,
    MustRepair,
}

#[allow(dead_code)]
impl Writer {
    pub fn new(
        wi: WriterIngredients,
        sender: Rc<UdpSender>,
        set_writer_nack_sender: mio_channel::Sender<(EntityId, GUID)>,
    ) -> Self {
        let mut msg = String::new();
        msg += "\tunicast locators\n";
        for loc in &wi.unicast_locator_list {
            msg += &format!("\t\t{}\n", loc);
        }
        msg += "\tmulticast locators\n";
        for loc in &wi.multicast_locator_list {
            msg += &format!("\t\t{}\n", loc);
        }
        info!(
            "created new Writer of Topic ({}, {}) with Locators\n{}\tWriter: {}",
            wi.topic.name(),
            wi.topic.type_desc(),
            msg,
            wi.guid,
        );
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
            last_change_sequence_number: SequenceNumber(0),
            writer_cache: Arc::new(RwLock::new(HistoryCache::new())),
            data_max_size_serialized: wi.data_max_size_serialized,
            reader_locators: Vec::new(),
            matched_readers: BTreeMap::new(),
            total_matched_readers: BTreeSet::new(),
            topic: wi.topic,
            qos: wi.qos,
            endianness: Endianness::LittleEndian,
            writer_command_receiver: wi.writer_command_receiver,
            writer_state_notifier: wi.writer_state_notifier,
            set_writer_nack_sender,
            sender,
            hb_counter: 0,
            an_state: AckNackState::Waiting,
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
        let ts = Timestamp::now().expect("failed get Timestamp::now()");
        CacheChange::new(
            kind,
            self.guid,
            self.last_change_sequence_number,
            ts,
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
            rl.unsent_changes = self.writer_cache.read().changes.values().cloned().collect();
        }
    }

    pub fn entity_token(&self) -> Token {
        self.entity_id().as_token()
    }

    pub fn handle_writer_cmd(&mut self) {
        while let Ok(cmd) = self.writer_command_receiver.try_recv() {
            match cmd {
                WriterCmd::WriteData(sp) => self.handle_write_data_cmd(sp),
                WriterCmd::AssertLiveliness => self.assert_liveliness_manually(),
            }
        }
    }

    pub fn assert_liveliness(&mut self) {
        todo!();
    }

    fn assert_liveliness_manually(&mut self) {
        // rtps 2.3 spec, 8.3.7.5 Heartbeat, Table 8.38 - Structure of the Heartbeat Submessage
        // > LivelinessFlag: Appears in the Submessage header flags. Indicates that the DDS DataWriter associated with the RTPS Writer of the message has manually asserted its LIVELINESS.
        self.send_heart_beat(true);
    }

    fn handle_write_data_cmd(&mut self, serialized_payload: Option<SerializedPayload>) {
        // this is new_change
        self.last_change_sequence_number += SequenceNumber(1);
        let ts = Timestamp::now().expect("failed get Timestamp::now()");
        let a_change = CacheChange::new(
            ChangeKind::Alive,
            self.guid,
            self.last_change_sequence_number,
            ts,
            serialized_payload,
            InstantHandle {},
        );
        // register a_change to writer HistoryCache
        if let Err(e) = self.add_change_to_hc(a_change.clone()) {
            debug!(
                "add_change to Reader failed: {}\n\tWriter: {}",
                e, self.guid
            );
            return;
        }
        let self_guid_prefix = self.guid_prefix();
        for reader_proxy in self.matched_readers.values_mut() {
            while let Some(change_for_reader) = reader_proxy.next_unsent_change() {
                reader_proxy.update_cache_state(
                    change_for_reader.seq_num,
                    change_for_reader._is_relevant,
                    ChangeForReaderStatusKind::Underway,
                );
                if change_for_reader._is_relevant {
                    if let Some(aa_change) = self
                        .writer_cache
                        .read()
                        .get_change(self.guid, change_for_reader.seq_num)
                    {
                        // build RTPS Message
                        let mut message_builder = MessageBuilder::new();
                        let time_stamp = Timestamp::now();
                        message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                        message_builder.data(
                            Endianness::LittleEndian,
                            self.guid.entity_id,
                            reader_proxy.remote_reader_guid.entity_id,
                            aa_change,
                        );
                        let message = message_builder.build(self_guid_prefix);
                        let message_buf = message
                            .write_to_vec_with_ctx(self.endianness)
                            .expect("couldn't serialize message");

                        // TODO:
                        // unicastとmulticastの両方に送信する必要はないから、状況によって切り替えるようにする。
                        let mut is_send = false;
                        for uni_loc in reader_proxy.get_unicast_locator_list() {
                            if uni_loc.kind == Locator::KIND_UDPV4 {
                                let port = uni_loc.port;
                                let addr = uni_loc.address;
                                info!(
                                    "Writer send data message to {}.{}.{}.{}:{}\n\tWriter: {}",
                                    addr[12], addr[13], addr[14], addr[15], port, self.guid,
                                );
                                self.sender.send_to_unicast(
                                    &message_buf,
                                    Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                    port as u16,
                                );
                                is_send = true;
                            }
                        }
                        for mul_loc in reader_proxy.get_multicast_locator_list() {
                            if mul_loc.kind == Locator::KIND_UDPV4 {
                                let port = mul_loc.port;
                                let addr = mul_loc.address;
                                info!(
                                    "Writer send data message to {}.{}.{}.{}:{}\n\tWriter: {}",
                                    addr[12], addr[13], addr[14], addr[15], port, self.guid,
                                );
                                self.sender.send_to_multicast(
                                    &message_buf,
                                    Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                    port as u16,
                                );
                                is_send = true;
                            }
                        }
                        if !is_send {
                            error!("attempt to send data, but not found UDP_V4 locator of Reader\n\tWriter: {}\n\tReader: {}", self.guid, reader_proxy.remote_reader_guid);
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
                    let mut is_send = false;
                    for uni_loc in reader_proxy.get_unicast_locator_list() {
                        if uni_loc.kind == Locator::KIND_UDPV4 {
                            let port = uni_loc.port;
                            let addr = uni_loc.address;
                            info!(
                                "Writer send data message to {}.{}.{}.{}:{}\n\tWriter: {}",
                                addr[12], addr[13], addr[14], addr[15], port, self.guid,
                            );
                            self.sender.send_to_unicast(
                                &message_buf,
                                Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                port as u16,
                            );
                            is_send = true;
                        }
                    }
                    for mul_loc in reader_proxy.get_multicast_locator_list() {
                        if mul_loc.kind == Locator::KIND_UDPV4 {
                            let port = mul_loc.port;
                            let addr = mul_loc.address;
                            info!(
                                "Writer send data message to {}.{}.{}.{}:{}\n\tWriter: {}",
                                addr[12], addr[13], addr[14], addr[15], port, self.guid,
                            );
                            self.sender.send_to_multicast(
                                &message_buf,
                                Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                port as u16,
                            );
                            is_send = true;
                        }
                    }
                    if !is_send {
                        error!("attempt to send data, but not found UDP_V4 locator of Reader\n\tWriter: {}\n\tReader: {}", self.guid, reader_proxy.remote_reader_guid);
                    }
                }
            }
        }
    }

    pub fn send_heart_beat(&mut self, liveliness: bool) {
        let time_stamp = Timestamp::now();
        let writer_cache = self.writer_cache.read();
        self.hb_counter += 1;
        let self_guid_prefix = self.guid_prefix();
        let self_entity_id = self.entity_id();
        for reader_proxy in self.matched_readers.values_mut() {
            let mut message_builder = MessageBuilder::new();
            message_builder.info_ts(Endianness::LittleEndian, time_stamp);
            if writer_cache.get_seq_num_min().0 <= 0 || writer_cache.get_seq_num_max().0 < 0 {
                continue;
            }
            message_builder.heartbeat(
                self.endianness,
                liveliness,
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
            let mut is_send = false;
            for uni_loc in reader_proxy.get_unicast_locator_list() {
                if uni_loc.kind == Locator::KIND_UDPV4 {
                    let port = uni_loc.port;
                    let addr = uni_loc.address;
                    info!(
                        "Writer send heartbeat message to {}.{}.{}.{}:{}\n\tWriter: {}",
                        addr[12], addr[13], addr[14], addr[15], port, self.guid,
                    );
                    self.sender.send_to_unicast(
                        &message_buf,
                        Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                        port as u16,
                    );
                    is_send = true;
                }
            }
            for mul_loc in reader_proxy.get_multicast_locator_list() {
                if mul_loc.kind == Locator::KIND_UDPV4 {
                    let port = mul_loc.port;
                    let addr = mul_loc.address;
                    info!(
                        "Writer send heartbeat message to {}.{}.{}.{}:{}\n\tWriter: {}",
                        addr[12], addr[13], addr[14], addr[15], port, self.guid,
                    );
                    self.sender.send_to_multicast(
                        &message_buf,
                        Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                        port as u16,
                    );
                    is_send = true;
                }
            }
            if !is_send {
                error!("attempt to send heartbeat, but not found UDP_V4 locator of Reader\n\tWriter: {}\n\tReader: {}", self.guid, reader_proxy.remote_reader_guid);
            }
        }
    }

    fn add_change_to_hc(&mut self, change: CacheChange) -> Result<(), String> {
        // add change to WriterHistoryCache & set status to Unset on each ReaderProxy
        self.writer_cache.write().add_change(change)?;
        for reader_proxy in self.matched_readers.values_mut() {
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
        Ok(())
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
        let self_guid_prefix = self.guid_prefix();
        let self_entity_id = self.entity_id();
        if let Some(reader_proxy) = self.matched_readers.get_mut(&reader_guid) {
            let mut resend_count = 0;
            while let Some(change) = reader_proxy.next_requested_change() {
                resend_count += 1;
                reader_proxy.update_cache_state(
                    change.seq_num,
                    change._is_relevant,
                    ChangeForReaderStatusKind::Underway,
                );
                if change._is_relevant {
                    if let Some(aa_change) = self
                        .writer_cache
                        .read()
                        .get_change(self.guid, change.seq_num)
                    {
                        // build RTPS Message
                        let mut message_builder = MessageBuilder::new();
                        let time_stamp = Timestamp::now();
                        message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                        message_builder.data(
                            Endianness::LittleEndian,
                            self.guid.entity_id,
                            reader_proxy.remote_reader_guid.entity_id,
                            aa_change,
                        );
                        let message = message_builder.build(self_guid_prefix);
                        let message_buf = message
                            .write_to_vec_with_ctx(self.endianness)
                            .expect("couldn't serialize message");

                        // TODO:
                        // unicastとmulticastの両方に送信する必要はないから、状況によって切り替えるようにする。
                        let mut is_resend = false;
                        for uni_loc in reader_proxy.get_unicast_locator_list() {
                            if uni_loc.kind == Locator::KIND_UDPV4 {
                                let port = uni_loc.port;
                                let addr = uni_loc.address;
                                info!(
                                    "Writer resend data message to {}.{}.{}.{}:{}\n\tWriter: {}",
                                    addr[12], addr[13], addr[14], addr[15], port, self.guid,
                                );
                                self.sender.send_to_unicast(
                                    &message_buf,
                                    Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                    port as u16,
                                );
                                is_resend = true;
                            }
                        }
                        for mul_loc in reader_proxy.get_multicast_locator_list() {
                            if mul_loc.kind == Locator::KIND_UDPV4 {
                                let port = mul_loc.port;
                                let addr = mul_loc.address;
                                info!(
                                    "Writer resend data message to {}.{}.{}.{}:{}\n\tWriter: {}",
                                    addr[12], addr[13], addr[14], addr[15], port, self.guid,
                                );
                                self.sender.send_to_multicast(
                                    &message_buf,
                                    Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                    port as u16,
                                );
                                is_resend = true;
                            }
                        }
                        if !is_resend {
                            error!("attempt to resend data, but not found UDP_V4 locator of Reader\n\tWriter: {}\n\tReader: {}", self.guid, reader_guid);
                        }
                    } else {
                        // rtps spec, 8.4.2.2.4 Writers must eventually respond to a negative acknowledgment (reliable only)
                        // send Heartbeat because, the sample is no longer avilable
                        self.hb_counter += 1;
                        let time_stamp = Timestamp::now();
                        let writer_cache = self.writer_cache.read();
                        let mut message_builder = MessageBuilder::new();
                        message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                        message_builder.heartbeat(
                            self.endianness,
                            false,
                            self_entity_id,
                            reader_proxy.remote_reader_guid.entity_id,
                            change.seq_num + SequenceNumber(1),
                            writer_cache.get_seq_num_max(),
                            self.hb_counter - 1,
                            false,
                        );
                        let msg = message_builder.build(self_guid_prefix);
                        let message_buf = msg
                            .write_to_vec_with_ctx(self.endianness)
                            .expect("couldn't serialize message");
                        let mut is_send = false;
                        for uni_loc in reader_proxy.get_unicast_locator_list() {
                            if uni_loc.kind == Locator::KIND_UDPV4 {
                                let port = uni_loc.port;
                                let addr = uni_loc.address;
                                info!(
                                    "Writer send heartbeat message to {}.{}.{}.{}:{}\n\tWriter: {}",
                                    addr[12], addr[13], addr[14], addr[15], port, self.guid
                                );
                                self.sender.send_to_unicast(
                                    &message_buf,
                                    Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                    port as u16,
                                );
                                is_send = true;
                            }
                        }
                        for mul_loc in reader_proxy.get_multicast_locator_list() {
                            if mul_loc.kind == Locator::KIND_UDPV4 {
                                let port = mul_loc.port;
                                let addr = mul_loc.address;
                                info!(
                                    "Writer send heartbeat message to {}.{}.{}.{}:{}\n\tWriter: {}",
                                    addr[12], addr[13], addr[14], addr[15], port, self.guid
                                );
                                self.sender.send_to_multicast(
                                    &message_buf,
                                    Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                    port as u16,
                                );
                                is_send = true;
                            }
                        }
                        if !is_send {
                            error!("attempt to send heartbeat, but not found UDP_V4 locator of Reader\n\tWriter: {}\n\tReader: {}", self.guid, reader_guid);
                        }
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
                    let mut is_send = false;
                    for uni_loc in reader_proxy.get_unicast_locator_list() {
                        if uni_loc.kind == Locator::KIND_UDPV4 {
                            let port = uni_loc.port;
                            let addr = uni_loc.address;
                            info!(
                                "Writer send gap message to {}.{}.{}.{}:{}\n\tWriter: {}",
                                addr[12], addr[13], addr[14], addr[15], port, self.guid,
                            );
                            self.sender.send_to_unicast(
                                &message_buf,
                                Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                port as u16,
                            );
                            is_send = true;
                        }
                    }
                    for mul_loc in reader_proxy.get_multicast_locator_list() {
                        if mul_loc.kind == Locator::KIND_UDPV4 {
                            let port = mul_loc.port;
                            let addr = mul_loc.address;
                            info!(
                                "Writer send gap message to {}.{}.{}.{}:{}\n\tWriter: {}",
                                addr[12], addr[13], addr[14], addr[15], port, self.guid,
                            );
                            self.sender.send_to_multicast(
                                &message_buf,
                                Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                port as u16,
                            );
                            is_send = true;
                        }
                    }
                    if !is_send {
                        error!("attempt to send gap, but not found UDP_V4 locator of Reader\n\tWriter: {}\n\tReader: {}", self.guid, reader_guid);
                    }
                };
            }
            if resend_count == 0 {
                error!("handle_nack_response_timeout called, but there is no change to handle\n\tWriter: {}\n\tReader: {}", self.guid, reader_guid);
            }
        } else {
            error!(
                "not found Reader which attempt to send nack response from Writer\n\tReader: {}\n\tWriter: {}",
                reader_guid,  self.guid
            );
        }
        self.an_state = AckNackState::Waiting;
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
    pub fn matched_reader_loolup(&self, guid: GUID) -> Option<ReaderProxy> {
        self.matched_readers.get(&guid).cloned()
    }

    fn matched_reader_unmatch(&mut self, guid: GUID) {
        self.matched_readers.remove(&guid);
        self.writer_state_notifier
            .send(DataWriterStatusChanged::LivelinessLost(
                LivelinessLostStatus::new(todo!(), 1, guid),
            ))
            .expect("couldn't send writer_state_notifier");
    }

    pub fn matched_reader_remove(&mut self, guid: GUID) {
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

    fn delete_reader_proxy(&mut self, guid_prefix: GuidPrefix) {
        let to_delete: Vec<GUID> = self
            .matched_readers
            .keys()
            .filter(|k| k.guid_prefix != guid_prefix)
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
    // This implementation spesific
    pub topic: Topic,
    pub qos: DataWriterQosPolicies,
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
    pub writer_state_notifier: mio_channel::Sender<DataWriterStatusChanged>,
}
pub enum WriterCmd {
    WriteData(Option<SerializedPayload>),
    AssertLiveliness,
}
