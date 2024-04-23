use crate::message::{
    message_builder::MessageBuilder,
    submessage::element::{Locator, SequenceNumber, SerializedPayload, Timestamp},
};
use crate::network::udp_sender::UdpSender;
use crate::policy::ReliabilityQosKind;
use crate::rtps::cache::{CacheChange, CacheData, ChangeKind, HistoryCache, InstantHandle};
use crate::rtps::reader_locator::ReaderLocator;
use crate::structure::{
    duration::Duration, entity::RTPSEntity, entity_id::EntityId, guid::GUID, topic_kind::TopicKind,
};
use bytes::Bytes;
use mio_extras::channel as mio_channel;
use mio_v06::Token;
use speedy::{Endianness, Writable};
use std::net::Ipv4Addr;
use std::rc::Rc;

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
    writer_cache: HistoryCache,
    data_max_size_serialized: i32,
    // StatelessWriter
    reader_locators: Vec<ReaderLocator>,
    // This implementation spesific
    endianness: Endianness,
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
    sender: Rc<UdpSender>,
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
            writer_cache: HistoryCache::new(),
            data_max_size_serialized: wi.data_max_size_serialized,
            reader_locators: Vec::new(),
            endianness: Endianness::LittleEndian,
            writer_command_receiver: wi.writer_command_receiver,
            sender,
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Option<CacheData>,
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
            rl.unsent_changes = self.writer_cache.changes.clone();
        }
    }

    pub fn entity_token(&self) -> Token {
        self.entity_id().as_token()
    }

    pub fn handle_writer_cmd(&mut self) {
        while let Ok(cmd) = self.writer_command_receiver.try_recv() {
            eprintln!(
                "~~~~~~~~~~~~~Writer entity: {:?} processed @writer",
                self.guid_prefix()
            );
            // this is new_change
            self.last_change_sequence_number += SequenceNumber(1);
            let cache_data = match &cmd.serialized_payload {
                Some(v) => CacheData::new(v.value.clone()),
                None => CacheData::new(Bytes::from("")),
            };
            let a_change = CacheChange::new(
                ChangeKind::Alive,
                self.guid,
                self.last_change_sequence_number,
                Some(cache_data),
                InstantHandle {},
            );
            // TODO: register a_change to writer HistoryCache
            // build RTPS Message
            let mut message_builder = MessageBuilder::new();
            let time_stamp = Timestamp::now();
            message_builder.info_ts(Endianness::LittleEndian, time_stamp);
            message_builder.data(
                Endianness::LittleEndian,
                EntityId::UNKNOW,
                self.guid.entity_id,
                a_change,
                cmd.serialized_payload,
            );
            let message = message_builder.build(self.guid_prefix());
            let message_buf = message.write_to_vec_with_ctx(self.endianness).unwrap();
            self.sender
                .send_to_multicast(&message_buf, Ipv4Addr::new(239, 255, 0, 1), 7400);
        }
    }

    pub fn handle_acknack(&mut self) {
        todo!(); // TODO
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
