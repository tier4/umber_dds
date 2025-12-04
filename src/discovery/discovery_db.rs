use crate::dds::qos::policy::LivelinessQosKind;
use crate::discovery::structure::data::SPDPdiscoveredParticipantData;
use crate::message::submessage::element::Timestamp;
use crate::structure::{GuidPrefix, GUID};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use awkernel_sync::{mcs::MCSNode, mutex::Mutex};
use core::time::Duration as CoreDuration;
use log::warn;

#[derive(Clone, Copy)]
pub enum EndpointState {
    Live(Timestamp),
    LivelinessLost,
    Unknown,
}

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
        inner.write_participant_ts(guid_prefix, timestamp)
    }

    pub fn check_participant_liveliness(
        &mut self,
        timestamp: Timestamp,
    ) -> (CoreDuration, Vec<GuidPrefix>) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.check_participant_liveliness(timestamp)
    }

    /// Write the time when liveliness of remote Writers with guid_prefix was last updated to the discovery_db.
    pub fn update_liveliness_with_guid_prefix(
        &mut self,
        guid_prefix: GuidPrefix,
        timestamp: Timestamp,
        liveliness_kind: LivelinessQosKind,
    ) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.update_liveliness_with_guid_prefix_with_kind(guid_prefix, timestamp, liveliness_kind)
    }

    /*
    pub fn write_local_reader(&mut self, guid: GUID, timestamp: Timestamp) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.write_local_reader(guid, timestamp)
    }
    */
    /// Write the time when liveliness of a local writer with guid was last updated to the discovery_db.
    pub fn write_local_writer(
        &mut self,
        guid: GUID,
        timestamp: Timestamp,
        liveliness_kind: LivelinessQosKind,
    ) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.write_local_writer(guid, timestamp, liveliness_kind)
    }
    /*
    pub fn write_remote_reader(&mut self, guid: GUID, timestamp: Timestamp) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.write_remote_reader(guid, timestamp)
    }
    */
    /// Write the time when livelienss of a remote writer with guid was last updated to the discovery_db.
    pub fn write_remote_writer(
        &mut self,
        guid: GUID,
        timestamp: Timestamp,
        liveliness_kind: LivelinessQosKind,
    ) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.write_remote_writer(guid, timestamp, liveliness_kind)
    }

    pub fn update_remote_writer_state(&mut self, guid: GUID, state: EndpointState) {
        let mut node = MCSNode::new();
        let mut inner = self.inner.lock(&mut node);
        inner.update_remote_writer_state(guid, state)
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
    pub fn read_local_writer(&self, guid: GUID) -> EndpointState {
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
    pub fn read_remote_writer(&self, guid: GUID) -> EndpointState {
        let mut node = MCSNode::new();
        let inner = self.inner.lock(&mut node);
        inner.read_remote_writer(guid)
    }
}

struct DiscoveryDBInner {
    participant_data: BTreeMap<GuidPrefix, (EndpointState, SPDPdiscoveredParticipantData)>,
    // local_reader_data: BTreeMap<GUID, Timestamp>,
    local_writer_data: BTreeMap<GUID, (EndpointState, LivelinessQosKind)>,
    // remote_reader_data: BTreeMap<GUID, Timestamp>,
    remote_writer_data: BTreeMap<GUID, (EndpointState, LivelinessQosKind)>,
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
        self.participant_data
            .insert(guid_prefix, (EndpointState::Live(timestamp), data));
    }

    fn write_participant_ts(&mut self, guid_prefix: GuidPrefix, timestamp: Timestamp) {
        if let Some(e) = self.participant_data.get_mut(&guid_prefix) {
            e.0 = EndpointState::Live(timestamp);
        }
    }

    pub fn check_participant_liveliness(
        &mut self,
        timestamp: Timestamp,
    ) -> (CoreDuration, Vec<GuidPrefix>) {
        let mut next_duration = CoreDuration::MAX;
        let mut lost = Vec::new();
        for (prefix, (es, data)) in &mut self.participant_data {
            if let EndpointState::Live(ts) = es {
                if timestamp - *ts > data.lease_duration {
                    warn!(
                        "checked Liveliness of Participant Lost\n\tParticipant: {}",
                        prefix
                    );
                    lost.push(*prefix);
                } else {
                    next_duration = data.lease_duration.half().to_core_duration();
                }
            }
        }
        for l in &lost {
            self.participant_data.remove(l);
        }
        if next_duration == CoreDuration::MAX {
            (CoreDuration::new(5, 0), lost)
        } else {
            (next_duration, lost)
        }
    }

    fn update_liveliness_with_guid_prefix_with_kind(
        &mut self,
        guid_prefix: GuidPrefix,
        timestamp: Timestamp,
        liveliness_kind: LivelinessQosKind,
    ) {
        let to_update: Vec<(GUID, LivelinessQosKind)> = self
            .remote_writer_data
            .iter()
            .filter(|&(k, v)| k.guid_prefix == guid_prefix && v.1 == liveliness_kind)
            .map(|(k, v)| (*k, v.1))
            .collect();
        for (w_guid, liveliness_kind) in to_update {
            self.remote_writer_data
                .insert(w_guid, (EndpointState::Live(timestamp), liveliness_kind));
        }
    }

    fn update_liveliness_with_guid_prefix(
        &mut self,
        guid_prefix: GuidPrefix,
        timestamp: Timestamp,
    ) {
        let to_update: Vec<(GUID, LivelinessQosKind)> = self
            .remote_writer_data
            .iter()
            .filter(|&(k, _v)| k.guid_prefix == guid_prefix)
            .map(|(k, v)| (*k, v.1))
            .collect();
        for (w_guid, liveliness_kind) in to_update {
            self.remote_writer_data
                .insert(w_guid, (EndpointState::Live(timestamp), liveliness_kind));
        }
    }

    /*
    fn write_local_reader(&mut self, guid: GUID, timestamp: Timestamp) {
        self.local_reader_data.insert(guid, timestamp);
    }
    */
    fn write_local_writer(
        &mut self,
        guid: GUID,
        timestamp: Timestamp,
        // is_manual_by_participant: bool,
        liveliness_kind: LivelinessQosKind,
    ) {
        // DDS 1.4 spec, 2.2.3.11 LIVELINESS
        // The setting MANUAL_BY_PARTICIPANT requires only that one Entity within the publisher is asserted to be alive to deduce all other Entity objects within the same DomainParticipant are also alive.
        // if is_manual_by_participant {
        self.local_writer_data
            .insert(guid, (EndpointState::Live(timestamp), liveliness_kind));
        if liveliness_kind == LivelinessQosKind::ManualByParticipant {
            self.update_liveliness_with_guid_prefix(guid.guid_prefix, timestamp)
        }
    }
    /*
    fn write_remote_reader(&mut self, guid: GUID, timestamp: Timestamp) {
        self.remote_reader_data.insert(guid, timestamp);
    }
    */
    fn write_remote_writer(
        &mut self,
        guid: GUID,
        timestamp: Timestamp,
        // is_manual_by_participant: bool,
        liveliness_kind: LivelinessQosKind,
    ) {
        // DDS 1.4 spec, 2.2.3.11 LIVELINESS
        // The setting MANUAL_BY_PARTICIPANT requires only that one Entity within the publisher is asserted to be alive to deduce all other Entity objects within the same DomainParticipant are also alive.
        // if is_manual_by_participant {
        self.remote_writer_data
            .insert(guid, (EndpointState::Live(timestamp), liveliness_kind));
        if liveliness_kind == LivelinessQosKind::ManualByParticipant {
            self.update_liveliness_with_guid_prefix(guid.guid_prefix, timestamp)
        }
    }

    fn update_remote_writer_state(&mut self, guid: GUID, state: EndpointState) {
        if let Some((es, _l)) = self.remote_writer_data.get_mut(&guid) {
            *es = state;
        }
    }

    fn read_participant_data(
        &self,
        guid_prefix: GuidPrefix,
    ) -> Option<SPDPdiscoveredParticipantData> {
        if let Some((_ts, data)) = self.participant_data.get(&guid_prefix) {
            Some(data.clone())
        } else {
            None
        }
    }

    fn _read_participant_ts(&self, guid_prefix: GuidPrefix) -> Option<Timestamp> {
        if let Some((EndpointState::Live(ts), _data)) = self.participant_data.get(&guid_prefix) {
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
    fn read_local_writer(&self, guid: GUID) -> EndpointState {
        if let Some((es, _)) = self.local_writer_data.get(&guid) {
            *es
        } else {
            EndpointState::Unknown
        }
    }
    /*
    fn read_remote_reader(&self, guid: GUID) -> Option<Timestamp> {
        self.remote_reader_data.get(&guid).copied()
    }
    */
    fn read_remote_writer(&self, guid: GUID) -> EndpointState {
        if let Some((es, _)) = self.remote_writer_data.get(&guid) {
            *es
        } else {
            EndpointState::Unknown
        }
    }
}
