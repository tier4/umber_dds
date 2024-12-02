use crate::discovery::structure::data::SPDPdiscoveredParticipantData;
use crate::message::submessage::element::Timestamp;
use crate::structure::GuidPrefix;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use std::sync::Mutex;

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

    pub fn write(
        &mut self,
        guid_prefix: GuidPrefix,
        timestamp: Timestamp,
        data: SPDPdiscoveredParticipantData,
    ) {
        let mut inner = self.inner.lock().expect("couldn't lock DiscoveryDBInner");
        inner.write(guid_prefix, timestamp, data)
    }

    pub fn read_data(&self, guid_prefix: GuidPrefix) -> Option<SPDPdiscoveredParticipantData> {
        let inner = self.inner.lock().expect("couldn't lock DiscoveryDBInner");
        inner.read_data(guid_prefix)
    }

    pub fn read_ts(&self, guid_prefix: GuidPrefix) -> Option<Timestamp> {
        let inner = self.inner.lock().expect("couldn't lock DiscoveryDBInner");
        inner.read_ts(guid_prefix)
    }
}

struct DiscoveryDBInner {
    data: BTreeMap<GuidPrefix, (Timestamp, SPDPdiscoveredParticipantData)>,
}

impl DiscoveryDBInner {
    fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    fn write(
        &mut self,
        guid_prefix: GuidPrefix,
        timestamp: Timestamp,
        data: SPDPdiscoveredParticipantData,
    ) {
        self.data.insert(guid_prefix, (timestamp, data));
    }

    fn read_data(&self, guid_prefix: GuidPrefix) -> Option<SPDPdiscoveredParticipantData> {
        if let Some((_ts, data)) = self.data.get(&guid_prefix) {
            Some((*data).clone())
        } else {
            None
        }
    }

    fn read_ts(&self, guid_prefix: GuidPrefix) -> Option<Timestamp> {
        if let Some((ts, _data)) = self.data.get(&guid_prefix) {
            Some(*ts)
        } else {
            None
        }
    }
}
