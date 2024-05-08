use crate::message::submessage::element::{Locator, SequenceNumber};
use crate::rtps::cache::{ChangeForReader, ChangeForReaderStatusKind, HistoryCache};
use crate::structure::{guid::GUID, parameter_id::ParameterId};
use serde::{ser::SerializeStruct, Serialize};
use std::cmp::min;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct ReaderProxy {
    pub remote_reader_guid: GUID,
    pub expects_inline_qos: bool,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    history_cache: Arc<RwLock<HistoryCache>>,
    cache_state: HashMap<SequenceNumber, ChangeForReader>,
}

impl ReaderProxy {
    pub fn new(
        remote_reader_guid: GUID,
        expects_inline_qos: bool,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        history_cache: Arc<RwLock<HistoryCache>>,
    ) -> Self {
        Self {
            remote_reader_guid,
            expects_inline_qos,
            unicast_locator_list,
            multicast_locator_list,
            history_cache,
            cache_state: HashMap::new(),
        }
    }
    pub fn acked_changes_set(&mut self, commited_seq_num: SequenceNumber) {
        let hc = self.history_cache.read().unwrap();
        for (k, v) in &hc.changes {
            if v.sequence_number <= commited_seq_num {
                self.cache_state.insert(
                    *k,
                    ChangeForReader::new(*k, ChangeForReaderStatusKind::Acknowledged, false),
                );
            }
        }
    }
    pub fn next_requested_change(&self) -> Option<ChangeForReader> {
        let mut next_seq_num = SequenceNumber::MAX;
        for change in self.requested_changes() {
            next_seq_num = min(next_seq_num, change.seq_num);
        }

        for change in self.requested_changes() {
            if change.seq_num == next_seq_num {
                return Some(change);
            }
        }
        None
    }
    pub fn next_unsent_change(&self) -> Option<ChangeForReader> {
        let mut next_seq_num = SequenceNumber::MAX;

        for change in self.unsent_changes() {
            next_seq_num = min(next_seq_num, change.seq_num);
        }

        for change in self.unsent_changes() {
            if change.seq_num == next_seq_num {
                return Some(change);
            }
        }
        None
    }
    pub fn requested_changes(&self) -> Vec<ChangeForReader> {
        let mut requested_changes = Vec::new();
        for (_sn, change_for_reader) in &self.cache_state {
            match change_for_reader.status {
                ChangeForReaderStatusKind::Unsent => {
                    requested_changes.push(change_for_reader.clone())
                }
                _ => (),
            }
        }
        requested_changes
    }
    pub fn requested_changes_set(&mut self, req_seq_num_set: Vec<SequenceNumber>) {
        for seq_num in req_seq_num_set {
            for (sn, change_for_reader) in &mut self.cache_state {
                if *sn == seq_num {
                    change_for_reader.status = ChangeForReaderStatusKind::Requested;
                }
            }
        }
    }
    pub fn unsent_changes(&self) -> Vec<ChangeForReader> {
        let mut unsent_changes = Vec::new();
        for (_sn, change_for_reader) in &self.cache_state {
            match change_for_reader.status {
                ChangeForReaderStatusKind::Unsent => unsent_changes.push(change_for_reader.clone()),
                _ => (),
            }
        }
        unsent_changes
    }
    pub fn unacked_changes(&self) -> Vec<ChangeForReader> {
        let mut unacked_changes = Vec::new();
        for (_sn, change_for_reader) in &self.cache_state {
            match change_for_reader.status {
                ChangeForReaderStatusKind::Unsent => {
                    unacked_changes.push(change_for_reader.clone())
                }
                _ => (),
            }
        }
        unacked_changes
    }
}
impl Serialize for ReaderProxy {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ReaderProxy", 4)?;
        // remote_reader_guid
        s.serialize_field("parameterId", &ParameterId::PID_ENDPOINT_GUID.value)?;
        s.serialize_field::<u16>("parameterLength", &16)?;
        s.serialize_field("remote_reader_guid", &self.remote_reader_guid)?;

        // expects_inline_qos
        s.serialize_field("parameterId", &ParameterId::PID_EXPECTS_INLINE_QOS.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("expects_inline_qos", &self.expects_inline_qos)?;
        s.serialize_field::<u16>("padding", &0)?;

        // unicast_locator_list
        for unicast_locator in &self.unicast_locator_list {
            s.serialize_field("parameterId", &ParameterId::PID_UNICAST_LOCATOR.value)?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field("unicast_locator_list", &unicast_locator)?;
        }

        // multicast_locator_list
        for multicast_locator in &self.multicast_locator_list {
            s.serialize_field("parameterId", &ParameterId::PID_MULTICAST_LOCATOR.value)?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field("multicast_locator_list", &multicast_locator)?;
        }

        s.end()
    }
}

#[derive(Clone)]
pub struct WriterProxy {
    pub remote_writer_guid: GUID,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub data_max_size_serialized: i32, // in rtps 2.3 spec, Figure 8.30: long
}

impl WriterProxy {
    pub fn new(
        remote_writer_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: i32,
    ) -> Self {
        Self {
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
        }
    }
}
impl Serialize for WriterProxy {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("WriterProxy", 4)?;
        // remote_writer_guid
        s.serialize_field("parameterId", &ParameterId::PID_ENDPOINT_GUID.value)?;
        s.serialize_field::<u16>("parameterLength", &16)?;
        s.serialize_field("remote_reader_guid", &self.remote_writer_guid)?;

        // unicast_locator_list
        for unicast_locator in &self.unicast_locator_list {
            s.serialize_field("parameterId", &ParameterId::PID_UNICAST_LOCATOR.value)?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field("unicast_locator_list", &unicast_locator)?;
        }

        // multicast_locator_list
        for multicast_locator in &self.multicast_locator_list {
            s.serialize_field("parameterId", &ParameterId::PID_MULTICAST_LOCATOR.value)?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field("multicast_locator_list", &multicast_locator)?;
        }

        // data_max_size_serialized
        s.serialize_field(
            "parameterId",
            &ParameterId::PID_TYPE_MAX_SIZE_SERIALIZED.value,
        )?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("data_max_size_serialized", &self.data_max_size_serialized)?;

        s.end()
    }
}
