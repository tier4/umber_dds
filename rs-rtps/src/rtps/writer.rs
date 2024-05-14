use crate::message::{
    message_builder::MessageBuilder,
    submessage::element::{Count, Locator, SequenceNumber, SerializedPayload, Timestamp},
};
use crate::network::udp_sender::UdpSender;
use crate::policy::ReliabilityQosKind;
use crate::rtps::cache::{
    CacheChange, ChangeForReaderStatusKind, ChangeKind, HistoryCache, InstantHandle,
};
use crate::rtps::reader_locator::ReaderLocator;
use crate::structure::{
    duration::Duration, entity::RTPSEntity, entity_id::EntityId, guid::GUID, proxy::ReaderProxy,
    topic_kind::TopicKind,
};
use mio_extras::channel as mio_channel;
use mio_v06::Token;
use speedy::{Endianness, Writable};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

/// RTPS StatelessWriter
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
    nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    writer_cache: Arc<RwLock<HistoryCache>>,
    data_max_size_serialized: i32,
    // StatelessWriter
    reader_locators: Vec<ReaderLocator>,
    // StatefulWriter
    reader_proxy: HashMap<GUID, ReaderProxy>,
    // This implementation spesific
    endianness: Endianness,
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
    sender: Rc<UdpSender>,
    hb_counter: Count,
}

impl Writer {
    pub fn new(wi: WriterIngredients, sender: Rc<UdpSender>) -> Self {
        Self {
            guid: wi.guid,
            topic_kind: wi.topic_kind,
            reliability_level: wi.reliability_level,
            unicast_locator_list: wi.unicast_locator_list,
            multicast_locator_list: wi.multicast_locator_list,
            push_mode: wi.push_mode,
            heartbeat_period: wi.heartbeat_period,
            nack_response_delay: wi.nack_response_delay,
            nack_suppression_duration: wi.nack_suppression_duration,
            last_change_sequence_number: SequenceNumber(0),
            writer_cache: Arc::new(RwLock::new(HistoryCache::new())),
            data_max_size_serialized: wi.data_max_size_serialized,
            reader_locators: Vec::new(),
            reader_proxy: HashMap::new(),
            endianness: Endianness::LittleEndian,
            writer_command_receiver: wi.writer_command_receiver,
            sender,
            hb_counter: 0,
        }
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
                .unwrap()
                .changes
                .values()
                .cloned()
                .collect();
        }
    }

    pub fn entity_token(&self) -> Token {
        self.entity_id().as_token()
    }

    pub fn handle_writer_cmd(&mut self) {
        while let Ok(cmd) = self.writer_command_receiver.try_recv() {
            eprintln!(
                "~~~~~~~~~~~~~Writer entity {:?}: processed @writer",
                self.guid().entity_id
            );
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
            let self_entity_id = self.guid().entity_id;
            let self_guid_prefix = self.guid_prefix();
            for (_guid, reader_proxy) in &mut self.reader_proxy {
                eprintln!(
                    "~~~~~~~~~~~~~Writer entity {:?}: for remote reader {:?} @writer",
                    self_entity_id, reader_proxy.remote_reader_guid
                );
                while let Some(change_for_reader) = reader_proxy.next_unsent_change() {
                    eprintln!(
                        "~~~~~~~~~~~~~Writer entity {:?}: find unsent change  @writer",
                        self_entity_id
                    );
                    reader_proxy.update_cache_state(
                        change_for_reader.seq_num,
                        change_for_reader.is_relevant,
                        ChangeForReaderStatusKind::Underway,
                    );
                    if change_for_reader.is_relevant {
                        if let Some(aa_change) = self
                            .writer_cache
                            .read()
                            .unwrap()
                            .get_change(change_for_reader.seq_num)
                        {
                            // build RTPS Message
                            let mut message_builder = MessageBuilder::new();
                            let time_stamp = Timestamp::now();
                            message_builder.info_ts(Endianness::LittleEndian, time_stamp);
                            message_builder.data(
                                Endianness::LittleEndian,
                                EntityId::UNKNOW,
                                self.guid.entity_id,
                                aa_change,
                            );
                            let message = message_builder.build(self_guid_prefix);
                            let message_buf =
                                message.write_to_vec_with_ctx(self.endianness).unwrap();

                            eprintln!(
                                "~~~~~~~~~~~~~Writer entity {:?}: send unsent change  @writer",
                                self_entity_id
                            );
                            // TODO:
                            // unicastとmulticastの両方に送信する必要はないから、状況によって切り替えるようにする。
                            for uni_loc in &reader_proxy.unicast_locator_list {
                                if uni_loc.kind == Locator::KIND_UDPV4 {
                                    let port = uni_loc.port;
                                    let addr = uni_loc.address;
                                    eprintln!(
                                        "~~~~~~~~~~~~~Writer entity {:?}: send to {}.{}.{}.{}:{}  @writer",
                                        self_entity_id, addr[12], addr[13], addr[14], addr[15], port);
                                    self.sender.send_to(
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
                                        "~~~~~~~~~~~~~Writer entity {:?}: send to {}.{}.{}.{}:{}  @writer",
                                        self_entity_id, addr[12], addr[13], addr[14], addr[15], port);
                                    self.sender.send_to(
                                        &message_buf,
                                        Ipv4Addr::new(addr[12], addr[13], addr[14], addr[15]),
                                        port as u16,
                                    );
                                }
                            }
                        } else {
                            todo!(); // TODO: send GAP
                        }
                    } else {
                        unreachable!();
                    }
                }
            }
        }
    }

    pub fn send_heart_beat(&mut self) {
        let mut message_builder = MessageBuilder::new();
        let time_stamp = Timestamp::now();
        message_builder.info_ts(Endianness::LittleEndian, time_stamp);
        let writer_cache = self.writer_cache.read().unwrap();
        message_builder.heartbeat(
            self.endianness,
            self.entity_id(),
            self.entity_id(),
            writer_cache.get_seq_num_min(),
            writer_cache.get_seq_num_max(),
            self.hb_counter,
            false,
        );
        self.hb_counter += 1;
        let msg = message_builder.build(self.guid_prefix());
        let message_buf = msg.write_to_vec_with_ctx(self.endianness).unwrap();
        for (_guid, reader_proxy) in &mut self.reader_proxy {
            for uni_loc in &reader_proxy.unicast_locator_list {
                if uni_loc.kind == Locator::KIND_UDPV4 {
                    let port = uni_loc.port;
                    let addr = uni_loc.address;
                    self.sender.send_to(
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
                    self.sender.send_to(
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
        self.writer_cache.write().unwrap().add_change(change);
        for (_guid, reader_proxy) in &mut self.reader_proxy {
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

    pub fn handle_acknack(&mut self) {
        todo!(); // TODO
    }

    pub fn matched_reader_add(
        &mut self,
        remote_reader_guid: GUID,
        expects_inline_qos: bool,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
    ) {
        self.reader_proxy.insert(
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
    pub fn matched_reader_loolup(&self, guid: GUID) -> Option<ReaderProxy> {
        match self.reader_proxy.get(&guid) {
            Some(prxy) => Some(prxy.clone()),
            None => None,
        }
    }
    pub fn matched_reader_remove(&mut self, guid: GUID) {
        self.reader_proxy.remove(&guid);
    }

    pub fn heartbeat_period(&self) -> std::time::Duration {
        std::time::Duration::new(
            self.heartbeat_period.seconds as u64,
            self.heartbeat_period.fraction,
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
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
}
pub struct WriterCmd {
    pub serialized_payload: Option<SerializedPayload>,
}
