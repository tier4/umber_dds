use crate::discovery::structure::data::SPDPdiscoveredParticipantData;
use crate::message::submessage::element::Timestamp;
use crate::structure::{GuidPrefix, GUID};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use awkernel_sync::{mcs::MCSNode, mutex::Mutex};

/// DiscoveryDB has following three purposes
/// 1. Manege remote Participant data.
/// 2. Manege liveliness of Participant.
/// 3. Manege Writer Liveliness.
#[derive(Clone)]
pub struct DiscoveryDB {
    inner: Arc<Mutex<DiscoveryDBInner>>,
}
impl DiscoveryDB {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DiscoveryDBInner::new())),
        }
    }

    pub fn write_participant(
        &mut self,
        guid_prefix: GuidPrefix,
        timestamp: Timestamp,
        data: SPDPdiscoveredParticipantData,
    ) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.write_participant(guid_prefix, timestamp, data)
    }

    /// Write the time when liveliness of Participant with guid_prefix was last updated to the discovery_db.
    pub fn write_participant_ts(&mut self, guid_prefix: GuidPrefix, timestamp: Timestamp) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        if let Some(data) = inner.read_participant_data(guid_prefix) {
            inner.write_participant(guid_prefix, timestamp, data)
        }
    }

    /// Write the time when liveliness of remote Writers with guid_prefix was last updated to the discovery_db.
    pub fn update_liveliness_with_guid_prefix(
        &mut self,
        guid_prefix: GuidPrefix,
        timestamp: Timestamp,
    ) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.update_liveliness_with_guid_prefix(guid_prefix, timestamp)
    }

    /*
    pub fn write_local_reader(&mut self, guid: GUID, timestamp: Timestamp) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.write_local_reader(guid, timestamp)
    }
    */
    /// Write the time when liveliness of a local writer with guid was last updated to the discovery_db.
    pub fn write_local_writer(&mut self, guid: GUID, timestamp: Timestamp) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.write_local_writer(guid, timestamp)
    }
    /*
    pub fn write_remote_reader(&mut self, guid: GUID, timestamp: Timestamp) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.write_remote_reader(guid, timestamp)
    }
    */
    /// Write the time when livelienss of a remote writer with guid was last updated to the discovery_db.
    pub fn write_remote_writer(&mut self, guid: GUID, timestamp: Timestamp) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.write_remote_writer(guid, timestamp)
    }

    pub fn read_participant_data(
        &self,
        guid_prefix: GuidPrefix,
    ) -> Option<SPDPdiscoveredParticipantData> {
        let mut node = MCSNode::new();
        let inner = self.inner.lock(&mut node);
        inner.read_participant_data(guid_prefix)
    }

    pub fn _read_participant_ts(&self, guid_prefix: GuidPrefix) -> Option<Timestamp> {
        let mut node = MCSNode::new();
        let inner = self.inner.lock(&mut node);
        inner._read_participant_ts(guid_prefix)
    }

    /*
    pub fn read_local_reader(&self, guid: GUID) -> Option<Timestamp> {
        let mut node = MCSNode::new();
        let inner = self.inner.lock(&mut node);
        inner.read_local_reader(guid)
    }
    */
    /// Read the time when livelienss of a local writer with guid was last updated from the discovery_db.
    pub fn read_local_writer(&self, guid: GUID) -> Option<Timestamp> {
        let mut node = MCSNode::new();
        let inner = self.inner.lock(&mut node);
        inner.read_local_writer(guid)
    }
    /*
    pub fn read_remote_reader(&self, guid: GUID) -> Option<Timestamp> {
        let mut node = MCSNode::new();
        let inner = self.inner.lock(&mut node);
        inner.read_remote_reader(guid)
    }
    */
    /// Read the time when livelienss of a remote writer with guid was last updated from the discovery_db.
    pub fn read_remote_writer(&self, guid: GUID) -> Option<Timestamp> {
        let mut node = MCSNode::new();
        let inner = self.inner.lock(&mut node);
        inner.read_remote_writer(guid)
    }
}

struct DiscoveryDBInner {
    participant_data: BTreeMap<GuidPrefix, (Timestamp, SPDPdiscoveredParticipantData)>,
    // local_reader_data: BTreeMap<GUID, Timestamp>,
    local_writer_data: BTreeMap<GUID, Timestamp>,
    // remote_reader_data: BTreeMap<GUID, Timestamp>,
    remote_writer_data: BTreeMap<GUID, Timestamp>,
}

impl DiscoveryDBInner {
    fn new() -> Self {
        Self {
            participant_data: BTreeMap::new(),
            // local_reader_data: BTreeMap::new(),
            local_writer_data: BTreeMap::new(),
            // remote_reader_data: BTreeMap::new(),
            remote_writer_data: BTreeMap::new(),
        }
    }

    fn write_participant(
        &mut self,
        guid_prefix: GuidPrefix,
        timestamp: Timestamp,
        data: SPDPdiscoveredParticipantData,
    ) {
        self.participant_data.insert(guid_prefix, (timestamp, data));
    }

    fn update_liveliness_with_guid_prefix(
        &mut self,
        guid_prefix: GuidPrefix,
        timestamp: Timestamp,
    ) {
        let to_update: Vec<GUID> = self
            .remote_writer_data
            .iter()
            .filter(|&(k, _v)| k.guid_prefix == guid_prefix)
            .map(|(k, _v)| *k)
            .collect();
        for w_guid in to_update {
            self.remote_writer_data.insert(w_guid, timestamp);
        }
    }

    /*
    fn write_local_reader(&mut self, guid: GUID, timestamp: Timestamp) {
        self.local_reader_data.insert(guid, timestamp);
    }
    */
    fn write_local_writer(&mut self, guid: GUID, timestamp: Timestamp) {
        self.local_writer_data.insert(guid, timestamp);
    }
    /*
    fn write_remote_reader(&mut self, guid: GUID, timestamp: Timestamp) {
        self.remote_reader_data.insert(guid, timestamp);
    }
    */
    fn write_remote_writer(&mut self, guid: GUID, timestamp: Timestamp) {
        self.remote_writer_data.insert(guid, timestamp);
    }

    fn read_participant_data(
        &self,
        guid_prefix: GuidPrefix,
    ) -> Option<SPDPdiscoveredParticipantData> {
        if let Some((_ts, data)) = self.participant_data.get(&guid_prefix) {
            Some((*data).clone())
        } else {
            None
        }
    }

    fn _read_participant_ts(&self, guid_prefix: GuidPrefix) -> Option<Timestamp> {
        if let Some((ts, _data)) = self.participant_data.get(&guid_prefix) {
            Some(*ts)
        } else {
            None
        }
    }

    /*
    fn read_local_reader(&self, guid: GUID) -> Option<Timestamp> {
        self.local_reader_data.get(&guid).copied()
    }
    */
    fn read_local_writer(&self, guid: GUID) -> Option<Timestamp> {
        self.local_writer_data.get(&guid).copied()
    }
    /*
    fn read_remote_reader(&self, guid: GUID) -> Option<Timestamp> {
        self.remote_reader_data.get(&guid).copied()
    }
    */
    fn read_remote_writer(&self, guid: GUID) -> Option<Timestamp> {
        self.remote_writer_data.get(&guid).copied()
    }
}
