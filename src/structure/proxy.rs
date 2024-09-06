use crate::message::submessage::element::{Locator, SequenceNumber};
use crate::rtps::cache::{
    ChangeForReader, ChangeForReaderStatusKind, ChangeFromWriter, ChangeFromWriterStatusKind,
    HistoryCache,
};
use crate::structure::{guid::GUID, parameter_id::ParameterId};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use colored::*;
use core::cmp::{max, min};
use serde::{ser::SerializeStruct, Serialize};
use std::sync::RwLock;

#[derive(Clone)]
pub struct ReaderProxy {
    pub remote_reader_guid: GUID,
    pub expects_inline_qos: bool,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    history_cache: Arc<RwLock<HistoryCache>>,
    cache_state: BTreeMap<SequenceNumber, ChangeForReader>,
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
            cache_state: BTreeMap::new(),
        }
    }

    pub fn print_cache_states(&self) {
        eprintln!("<{}>: cache_state", "ReaderProxy: Info".green());
        for (sn, cfr) in self.cache_state.iter() {
            eprintln!(
                "\tsn: {}, state: {:?}, is_relevant: {}",
                sn.0, cfr.status, cfr.is_relevant
            );
        }
    }

    pub fn update_cache_state(
        &mut self,
        seq_num: SequenceNumber,
        is_relevant: bool,
        state: ChangeForReaderStatusKind,
    ) {
        // `is_relevant` is related to DDS_FILLTER
        // Now this implementation does not support DDS_FILLTER,
        // so is_relevant is always set to true.
        let change_for_reader = ChangeForReader::new(seq_num, state, is_relevant);
        self.cache_state.insert(seq_num, change_for_reader);
    }

    pub fn acked_changes_set(&mut self, commited_seq_num: SequenceNumber) {
        let hc = self
            .history_cache
            .read()
            .expect("couldn't read history_cache");
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
                ChangeForReaderStatusKind::Requested => {
                    requested_changes.push(change_for_reader.clone())
                }
                _ => (),
            }
        }
        requested_changes
    }
    pub fn requested_changes_set(&mut self, req_seq_num_set: Vec<SequenceNumber>) {
        for seq_num in req_seq_num_set {
            if let Some(cfr) = self.cache_state.get_mut(&seq_num) {
                cfr.status = ChangeForReaderStatusKind::Requested;
            } else {
                self.update_cache_state(seq_num, true, ChangeForReaderStatusKind::Requested);
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
    history_cache: Arc<RwLock<HistoryCache>>,
    cache_state: BTreeMap<SequenceNumber, ChangeFromWriter>,
}

impl WriterProxy {
    pub fn new(
        remote_writer_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: i32,
        history_cache: Arc<RwLock<HistoryCache>>,
    ) -> Self {
        Self {
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
            history_cache,
            cache_state: BTreeMap::new(),
        }
    }

    pub fn print_cache_states(&self) {
        eprintln!("<{}>: cache_state", "WriterProxy: Info".green());
        for (sn, cfw) in self.cache_state.iter() {
            eprintln!(
                "\tsn: {}, state: {:?}, is_relevant: {}",
                sn.0, cfw.status, cfw.is_relevant
            );
        }
    }

    fn update_cache_state(
        &mut self,
        seq_num: SequenceNumber,
        is_relevant: bool,
        state: ChangeFromWriterStatusKind,
    ) {
        // `is_relevant` is related to DDS_FILLTER
        // Now this implementation does not support DDS_FILLTER,
        // so is_relevant is always set to true.
        let change_from_writer = ChangeFromWriter::new(seq_num, state, is_relevant);
        self.cache_state.insert(seq_num, change_from_writer);
    }

    pub fn available_changes_max(&self) -> SequenceNumber {
        let mut max_seq_num = SequenceNumber(0);
        for (sn, cfw) in &self.cache_state {
            match cfw.status {
                ChangeFromWriterStatusKind::Received | ChangeFromWriterStatusKind::Lost => {
                    max_seq_num = max(max_seq_num, *sn);
                }
                _ => (),
            }
        }
        max_seq_num
    }
    pub fn irrelevant_change_set(&mut self, seq_num: SequenceNumber) {
        self.update_cache_state(seq_num, false, ChangeFromWriterStatusKind::Received);
    }
    pub fn lost_changes_update(&mut self, first_available_seq_num: SequenceNumber) {
        for (sn, cfw) in &mut self.cache_state {
            match cfw.status {
                ChangeFromWriterStatusKind::Uuknown | ChangeFromWriterStatusKind::Missing => {
                    if *sn < first_available_seq_num {
                        cfw.status = ChangeFromWriterStatusKind::Lost;
                    }
                }
                _ => (),
            }
        }
    }
    pub fn missing_changes(&self) -> Vec<SequenceNumber> {
        let mut missing_changes = Vec::new();
        for (sn, cfw) in &self.cache_state {
            match cfw.status {
                ChangeFromWriterStatusKind::Missing => missing_changes.push(*sn),
                _ => (),
            }
        }
        missing_changes
    }
    pub fn missing_changes_update(
        &mut self,
        first_available_seq_num: SequenceNumber,
        last_available_seq_num: SequenceNumber,
    ) {
        for sn in first_available_seq_num.0..=last_available_seq_num.0 {
            if let Some(cfw) = self.cache_state.get_mut(&SequenceNumber(sn)) {
                match cfw.status {
                    ChangeFromWriterStatusKind::Uuknown => {
                        if SequenceNumber(sn) <= last_available_seq_num {
                            cfw.status = ChangeFromWriterStatusKind::Missing;
                        }
                    }
                    _ => (),
                }
            } else {
                self.update_cache_state(
                    SequenceNumber(sn),
                    true,
                    ChangeFromWriterStatusKind::Missing,
                );
            }
        }
    }
    pub fn received_chage_set(&mut self, seq_num: SequenceNumber) {
        if let Some(cfw) = self.cache_state.get_mut(&seq_num) {
            cfw.status = ChangeFromWriterStatusKind::Received;
        } else {
            self.update_cache_state(seq_num, true, ChangeFromWriterStatusKind::Received);
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
