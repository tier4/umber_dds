use crate::discovery::structure::data::SPDPdiscoveredParticipantData;
use crate::message::submessage::element::Timestamp;
use crate::structure::{GuidPrefix, GUID};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use awkernel_sync::{mcs::MCSNode, mutex::Mutex};

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

    /*
    pub fn write_local_reader(&mut self, guid: GUID, timestamp: Timestamp) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.write_local_reader(guid, timestamp)
    }
    */
    /// Write the time when data was last sent by a local writer with a GUID to the discovery_db.
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
    /// Write the time when data was last received from a remote writer with a GUID to the discovery_db.
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
    /// Read the time when data was last sent by a local writer with a GUID from the discovery_db.
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
    /// Read the time when data was last sent by a remote writer with a GUID from the discovery_db.
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
