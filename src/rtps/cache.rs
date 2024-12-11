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
    pub last_added: BTreeMap<GUID, Timestamp>,
    pub min_seq_num: Option<SequenceNumber>,
    pub max_seq_num: Option<SequenceNumber>,
}

impl HistoryCache {
    pub fn new() -> Self {
        Self {
            changes: BTreeMap::new(),
            last_added: BTreeMap::new(),
            min_seq_num: None,
            max_seq_num: None,
        }
    }
    pub fn add_empty(&mut self, writer_guid: GUID, ts: Timestamp) {
        self.last_added.insert(writer_guid, ts);
    }
    pub fn add_change(&mut self, change: CacheChange) -> Result<(), ()> {
        let key = HCKey::new(change.writer_guid, change.sequence_number);
        if let Some(c) = self.changes.get(&key) {
            if c.data_value == change.data_value {
                Err(())
            } else {
                self.last_added.insert(key.guid, change.timestamp);
                self.changes.insert(key, change);
                Ok(())
            }
        } else {
            self.last_added.insert(key.guid, change.timestamp);
            self.changes.insert(key, change);
            Ok(())
        }
    }
    pub fn get_change(&self, guid: GUID, seq_num: SequenceNumber) -> Option<CacheChange> {
        self.changes.get(&HCKey::new(guid, seq_num)).cloned()
    }

    /// get the Timestamp of the last Change added to the HistoryCache from the Writer with the specified `writer_guid`.
    pub fn get_last_added_ts(&self, writer_guid: GUID) -> Option<&Timestamp> {
        self.last_added.get(&writer_guid)
    }

    pub fn get_alive_changes(&self) -> Vec<CacheChange> {
        self.changes
            .iter()
            .filter(|c| c.1.kind == ChangeKind::Alive)
            .map(|c| c.1.clone())
            .collect()
    }
    pub fn remove_change(&mut self, change: &CacheChange) {
        let mut to_del = Vec::new();
        for (k, v) in self.changes.iter() {
            if *v == *change {
                to_del.push(*k);
            }
        }
        for k in to_del {
            self.update_change_state(&k, ChangeKind::NotAliveDisposed);
        }
    }
    pub fn remove_notalive_changes(&mut self) {
        let to_del: Vec<HCKey> = self
            .changes
            .iter()
            .filter(|c| c.1.kind == ChangeKind::NotAliveDisposed)
            .map(|c| *c.0)
            .collect();
        for d in to_del {
            self.changes.remove(&d);
        }
        self.min_seq_num = None;
        self.max_seq_num = None;
    }
    pub fn get_seq_num_min(&self) -> SequenceNumber {
        let mut min = SequenceNumber::MAX;
        for k in self.changes.keys() {
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
        for k in self.changes.keys() {
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

    fn update_change_state(&mut self, key: &HCKey, kind: ChangeKind) {
        if let Some(todo_update) = self.changes.get_mut(key) {
            todo_update.kind = kind;
        }
    }
}

impl Default for HistoryCache {
    fn default() -> Self {
        Self::new()
    }
}
