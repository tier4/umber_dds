use crate::message::submessage::element::{SequenceNumber, SerializedPayload};
use crate::structure::GUID;
use alloc::collections::BTreeMap;

#[derive(PartialEq, Eq, Clone)]
pub struct CacheChange {
    kind: ChangeKind,
    pub writer_guid: GUID,
    pub sequence_number: SequenceNumber,
    data_value: Option<SerializedPayload>,
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
        data_value: Option<SerializedPayload>,
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

    pub fn data_value(&self) -> Option<SerializedPayload> {
        self.data_value.clone()
    }
}

#[derive(Clone, Debug)]
pub enum ChangeForReaderStatusKind {
    Unsent,
    Unacknowledged,
    Requested,
    Acknowledged,
    Underway,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ChangeFromWriterStatusKind {
    Lost,
    Missing,
    Received,
    Uuknown,
}

#[derive(Clone, Debug)]
pub struct ChangeForReader {
    pub seq_num: SequenceNumber,
    pub status: ChangeForReaderStatusKind,
    pub is_relevant: bool,
}

impl ChangeForReader {
    pub fn new(
        seq_num: SequenceNumber,
        status: ChangeForReaderStatusKind,
        is_relevant: bool,
    ) -> Self {
        Self {
            seq_num,
            status,
            is_relevant,
        }
    }
}

#[derive(Clone)]
pub struct ChangeFromWriter {
    pub _seq_num: SequenceNumber,
    pub is_relevant: bool,
    pub status: ChangeFromWriterStatusKind,
}

impl ChangeFromWriter {
    pub fn new(
        seq_num: SequenceNumber,
        status: ChangeFromWriterStatusKind,
        is_relevant: bool,
    ) -> Self {
        Self {
            _seq_num: seq_num,
            status,
            is_relevant,
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

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct InstantHandle {/* TODO */}

pub struct HistoryCache {
    pub changes: BTreeMap<SequenceNumber, CacheChange>,
    pub min_seq_num: Option<SequenceNumber>,
    pub max_seq_num: Option<SequenceNumber>,
}

impl HistoryCache {
    pub fn new() -> Self {
        Self {
            changes: BTreeMap::new(),
            min_seq_num: None,
            max_seq_num: None,
        }
    }
    pub fn add_change(&mut self, change: CacheChange) {
        self.changes.insert(change.sequence_number, change);
    }
    pub fn get_change(&self, seq_num: SequenceNumber) -> Option<CacheChange> {
        self.changes.get(&seq_num).cloned()
    }

    pub fn get_changes(&self) -> Vec<Option<SerializedPayload>> {
        self.changes.iter().map(|c| c.1.data_value()).collect()
    }
    pub fn remove_change(&mut self, change: CacheChange) {
        let mut to_del = Vec::new();
        for (k, v) in self.changes.iter() {
            if *v == change {
                to_del.push(*k);
            }
        }
        for k in to_del {
            self.changes.remove(&k);
        }
    }
    pub fn remove_changes(&mut self) {
        self.changes = BTreeMap::new();
        self.min_seq_num = None;
        self.max_seq_num = None;
    }
    pub fn get_seq_num_min(&self) -> SequenceNumber {
        let mut min = SequenceNumber::MAX;
        for c in &self.changes {
            if *c.0 < min {
                min = *c.0;
            }
        }
        if min == SequenceNumber::MAX {
            SequenceNumber(0)
        } else {
            min
        }
    }
    pub fn get_seq_num_max(&self) -> SequenceNumber {
        let mut max = SequenceNumber::MIN;
        for c in &self.changes {
            if *c.0 > max {
                max = *c.0;
            }
        }
        if max == SequenceNumber::MIN {
            SequenceNumber(0)
        } else {
            max
        }
    }
}

impl Default for HistoryCache {
    fn default() -> Self {
        Self::new()
    }
}
