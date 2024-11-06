use crate::message::submessage::element::{SequenceNumber, SerializedPayload, Timestamp};
use crate::structure::GUID;
use alloc::collections::BTreeMap;

#[derive(PartialEq, Eq, Clone)]
pub struct CacheChange {
    kind: ChangeKind,
    pub writer_guid: GUID,
    pub sequence_number: SequenceNumber,
    pub timestamp: Timestamp,
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
        timestamp: Timestamp,
        data_value: Option<SerializedPayload>,
        instance_handle: InstantHandle,
    ) -> Self {
        Self {
            kind,
            writer_guid,
            sequence_number,
            timestamp,
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
    pub _is_relevant: bool,
}

impl ChangeForReader {
    pub fn new(
        seq_num: SequenceNumber,
        status: ChangeForReaderStatusKind,
        _is_relevant: bool,
    ) -> Self {
        Self {
            seq_num,
            status,
            _is_relevant,
        }
    }
}

#[derive(Clone)]
pub struct ChangeFromWriter {
    pub _seq_num: SequenceNumber,
    pub _is_relevant: bool,
    pub status: ChangeFromWriterStatusKind,
}

impl ChangeFromWriter {
    pub fn new(
        seq_num: SequenceNumber,
        status: ChangeFromWriterStatusKind,
        _is_relevant: bool,
    ) -> Self {
        Self {
            _seq_num: seq_num,
            status,
            _is_relevant,
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct HCKey {
    pub guid: GUID,
    pub seq_num: SequenceNumber,
}
impl HCKey {
    pub fn new(guid: GUID, seq_num: SequenceNumber) -> Self {
        Self { guid, seq_num }
    }
}
impl PartialOrd for HCKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HCKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.seq_num
            .cmp(&other.seq_num)
            .then_with(|| self.guid.cmp(&other.guid))
    }
}

pub struct HistoryCache {
    pub changes: BTreeMap<HCKey, CacheChange>,
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
        self.changes.insert(
            HCKey::new(change.writer_guid, change.sequence_number),
            change,
        );
    }
    pub fn get_change(&self, guid: GUID, seq_num: SequenceNumber) -> Option<CacheChange> {
        self.changes.get(&HCKey::new(guid, seq_num)).cloned()
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
        for (k, _v) in &self.changes {
            if k.seq_num < min {
                min = k.seq_num;
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
        for (k, _v) in &self.changes {
            if k.seq_num > max {
                max = k.seq_num;
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
