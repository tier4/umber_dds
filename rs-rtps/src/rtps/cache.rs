use crate::message::submessage::element::SequenceNumber;
use crate::structure::guid::GUID;
use bytes::Bytes;

#[derive(PartialEq, Eq, Clone)]
pub struct CacheChange {
    kind: ChangeKind,
    writer_guid: GUID,
    pub sequence_number: SequenceNumber,
    data_value: Option<CacheData>,
    // inline_qos: ParameterList,
    instance_handle: InstantHandle, // In DDS, the value of the fields
                                    // labeled as ‘key’ within the data
                                    // uniquely identify each data-
                                    // object.
}

impl CacheChange {
    pub fn new(
        kind: ChangeKind,
        writer_guid: GUID,
        sequence_number: SequenceNumber,
        data_value: Option<CacheData>,
        instance_handle: InstantHandle,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            sequence_number,
            data_value,
            instance_handle,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAlive,
    NotAliveDisposed,
    NotAliveUnregistered,
}

#[derive(PartialEq, Eq, Clone)]
pub struct CacheData {
    data: Bytes,
}

impl CacheData {
    pub fn new(data: Bytes) -> Self {
        Self { data }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct InstantHandle {/* TODO */}

pub struct HistoryCache {
    pub changes: Vec<CacheChange>,
    pub min_seq_num: Option<SequenceNumber>,
    pub max_seq_num: Option<SequenceNumber>,
}

impl HistoryCache {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            min_seq_num: None,
            max_seq_num: None,
        }
    }
    pub fn add_change(&mut self, change: CacheChange) {
        self.changes.push(change);
    }
    pub fn remove_change(&mut self, change: CacheChange) {
        if let Some(remove_idx) = self.changes.iter().position(|c| *c == change) {
            self.changes.remove(remove_idx);
        }
    }
    pub fn get_seq_num_min(&self) -> SequenceNumber {
        let mut min = SequenceNumber::MAX;
        for c in &self.changes {
            if c.sequence_number < min {
                min = c.sequence_number;
            }
        }
        min
    }
    pub fn get_seq_num_max(&self) -> SequenceNumber {
        let mut max = SequenceNumber::MIN;
        for c in &self.changes {
            if c.sequence_number > max {
                max = c.sequence_number;
            }
        }
        max
    }
}
