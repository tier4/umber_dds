use crate::dds::qos::{DataReaderQosPolicies, DataWriterQosPolicies};
use crate::message::submessage::element::{Locator, SequenceNumber};
use crate::rtps::cache::{
    ChangeForReader, ChangeForReaderStatusKind, ChangeFromWriter, ChangeFromWriterStatusKind,
    HistoryCache,
};
use crate::structure::{guid::GUID, parameter_id::ParameterId};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use awkernel_sync::rwlock::RwLock;
use core::cmp::{max, min};
use serde::{ser::SerializeStruct, Serialize};

#[derive(Clone)]
pub struct ReaderProxy {
    pub remote_reader_guid: GUID,
    pub expects_inline_qos: bool,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub qos: DataReaderQosPolicies,
    _history_cache: Arc<RwLock<HistoryCache>>,
    cache_state: BTreeMap<SequenceNumber, ChangeForReader>,
}

impl ReaderProxy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        remote_reader_guid: GUID,
        expects_inline_qos: bool,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        qos: DataReaderQosPolicies,
        history_cache: Arc<RwLock<HistoryCache>>,
        push_mode: bool,
    ) -> Self {
        // Check whether the WriterProxy have at lease one Locator
        assert!(
            unicast_locator_list.len()
                + multicast_locator_list.len()
                + default_unicast_locator_list.len()
                + default_multicast_locator_list.len()
                > 0
        );
        let mut cache_state = BTreeMap::new();
        let status_kind = if push_mode {
            ChangeForReaderStatusKind::Unsent
        } else {
            ChangeForReaderStatusKind::Unacknowledged
        };
        {
            let hc = history_cache.read();
            for k in hc.changes.keys() {
                cache_state.insert(
                    k.seq_num,
                    ChangeForReader::new(k.seq_num, status_kind, true),
                );
            }
        }
        Self {
            remote_reader_guid,
            expects_inline_qos,
            unicast_locator_list,
            multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            qos,
            _history_cache: history_cache,
            cache_state,
        }
    }

    pub fn get_unicast_locator_list(&self) -> &Vec<Locator> {
        if self.unicast_locator_list.is_empty() {
            &self.default_unicast_locator_list
        } else {
            &self.unicast_locator_list
        }
    }

    pub fn get_multicast_locator_list(&self) -> &Vec<Locator> {
        if self.multicast_locator_list.is_empty() {
            &self.default_multicast_locator_list
        } else {
            &self.multicast_locator_list
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
        let mut to_update = Vec::new();
        for (seq_num, cfr) in &self.cache_state {
            if *seq_num <= commited_seq_num {
                to_update.push((*seq_num, cfr._is_relevant));
            }
        }
        for (seq_num, _is_relevant) in to_update {
            self.update_cache_state(
                seq_num,
                _is_relevant,
                ChangeForReaderStatusKind::Acknowledged,
            );
        }
    }
    pub fn next_requested_change(&self) -> Option<ChangeForReader> {
        let mut next_seq_num = SequenceNumber::MAX;
        for change in self.requested_changes() {
            next_seq_num = min(next_seq_num, change.seq_num);
        }

        self.requested_changes()
            .into_iter()
            .find(|change| change.seq_num == next_seq_num)
    }
    pub fn next_unsent_change(&self) -> Option<ChangeForReader> {
        let mut next_seq_num = SequenceNumber::MAX;

        for change in self.unsent_changes() {
            next_seq_num = min(next_seq_num, change.seq_num);
        }

        self.unsent_changes()
            .into_iter()
            .find(|change| change.seq_num == next_seq_num)
    }
    pub fn requested_changes(&self) -> Vec<ChangeForReader> {
        let mut requested_changes = Vec::new();
        for change_for_reader in self.cache_state.values() {
            if let ChangeForReaderStatusKind::Requested = change_for_reader.status {
                requested_changes.push(change_for_reader.clone())
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
        for change_for_reader in self.cache_state.values() {
            if let ChangeForReaderStatusKind::Unsent = change_for_reader.status {
                unsent_changes.push(change_for_reader.clone())
            }
        }
        unsent_changes
    }
    pub fn is_acked(&self) -> bool {
        let mut res = true;
        for change in self.cache_state.values() {
            res &= change._is_relevant;
            res &= change.status == ChangeForReaderStatusKind::Acknowledged;
        }
        res
    }
    #[allow(dead_code)]
    pub fn unacked_changes(&self) -> Vec<ChangeForReader> {
        let mut unacked_changes = Vec::new();
        for change_for_reader in self.cache_state.values() {
            if let ChangeForReaderStatusKind::Unsent = change_for_reader.status {
                unacked_changes.push(change_for_reader.clone())
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

        // durability
        s.serialize_field("parameterId", &ParameterId::PID_DURABILITY.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("durability", &self.qos.durability())?;

        // deadline
        s.serialize_field("parameterId", &ParameterId::PID_DEADLINE.value)?;
        s.serialize_field::<u16>("parameterLength", &8)?;
        s.serialize_field("deadline", &self.qos.deadline())?;

        // latency_budget
        s.serialize_field("parameterId", &ParameterId::PID_LATENCY_BUDGET.value)?;
        s.serialize_field::<u16>("parameterLength", &8)?;
        s.serialize_field("latency_budget", &self.qos.latency_budget())?;

        // liveliness
        s.serialize_field("parameterId", &ParameterId::PID_LIVELINESS.value)?;
        s.serialize_field::<u16>("parameterLength", &12)?;
        s.serialize_field("liveliness", &self.qos.liveliness())?;

        // reliability
        s.serialize_field("parameterId", &ParameterId::PID_RELIABILITY.value)?;
        s.serialize_field::<u16>("parameterLength", &12)?;
        s.serialize_field("reliability", &self.qos.reliability())?;

        // destination_order
        s.serialize_field("parameterId", &ParameterId::PID_DESTINATION_ORDER.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("reliability", &self.qos.destination_order())?;

        // history
        s.serialize_field("parameterId", &ParameterId::PID_HISTORY.value)?;
        s.serialize_field::<u16>("parameterLength", &8)?;
        s.serialize_field("reliability", &self.qos.history())?;

        // resouce_limits
        s.serialize_field("parameterId", &ParameterId::PID_RESOURCE_LIMITS.value)?;
        s.serialize_field::<u16>("parameterLength", &12)?;
        s.serialize_field("reliability", &self.qos.resource_limits())?;

        // user_data
        s.serialize_field("parameterId", &ParameterId::PID_USER_DATA.value)?;
        s.serialize_field::<u16>(
            "parameterLength",
            &(4 + self.qos.user_data().value.len() as u16),
        )?;
        s.serialize_field("user_data", &self.qos.user_data())?;

        // ownership
        s.serialize_field("parameterId", &ParameterId::PID_OWNERSHIP.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("ownership", &self.qos.ownership())?;

        // time_based_filter
        s.serialize_field("parameterId", &ParameterId::PID_TIME_BASED_FILTER.value)?;
        s.serialize_field::<u16>("parameterLength", &8)?;
        s.serialize_field("reliability", &self.qos.time_based_filter())?;

        s.end()
    }
}

#[derive(Clone)]
pub struct WriterProxy {
    pub remote_writer_guid: GUID,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub data_max_size_serialized: i32, // in rtps 2.3 spec, Figure 8.30: long
    pub qos: DataWriterQosPolicies,
    _history_cache: Arc<RwLock<HistoryCache>>,
    cache_state: BTreeMap<SequenceNumber, ChangeFromWriter>,
}

impl WriterProxy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        remote_writer_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: i32,
        qos: DataWriterQosPolicies,
        _history_cache: Arc<RwLock<HistoryCache>>,
    ) -> Self {
        // Check whether the WriterProxy have at lease one Locator
        assert!(
            unicast_locator_list.len()
                + multicast_locator_list.len()
                + default_unicast_locator_list.len()
                + default_multicast_locator_list.len()
                > 0
        );
        Self {
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            data_max_size_serialized,
            qos,
            _history_cache,
            cache_state: BTreeMap::new(),
        }
    }

    pub fn get_unicast_locator_list(&self) -> &Vec<Locator> {
        if self.unicast_locator_list.is_empty() {
            &self.default_unicast_locator_list
        } else {
            &self.unicast_locator_list
        }
    }

    pub fn get_multicast_locator_list(&self) -> &Vec<Locator> {
        if self.multicast_locator_list.is_empty() {
            &self.default_multicast_locator_list
        } else {
            &self.multicast_locator_list
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
            if let ChangeFromWriterStatusKind::Missing = cfw.status {
                missing_changes.push(*sn)
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
                if let ChangeFromWriterStatusKind::Uuknown = cfw.status {
                    if SequenceNumber(sn) <= last_available_seq_num {
                        cfw.status = ChangeFromWriterStatusKind::Missing;
                    }
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

        // durability
        s.serialize_field("parameterId", &ParameterId::PID_DURABILITY.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("durability", &self.qos.durability())?;

        // durability_service
        s.serialize_field("parameterId", &ParameterId::PID_DURABILITY_SERVICE.value)?;
        s.serialize_field::<u16>("parameterLength", &28)?;
        s.serialize_field("durability_service", &self.qos.durability_service())?;

        // deadline
        s.serialize_field("parameterId", &ParameterId::PID_DEADLINE.value)?;
        s.serialize_field::<u16>("parameterLength", &8)?;
        s.serialize_field("deadline", &self.qos.deadline())?;

        // latency_budget
        s.serialize_field("parameterId", &ParameterId::PID_LATENCY_BUDGET.value)?;
        s.serialize_field::<u16>("parameterLength", &8)?;
        s.serialize_field("latency_budget", &self.qos.latency_budget())?;

        // liveliness
        s.serialize_field("parameterId", &ParameterId::PID_LIVELINESS.value)?;
        s.serialize_field::<u16>("parameterLength", &12)?;
        s.serialize_field("liveliness", &self.qos.liveliness())?;

        // reliability
        s.serialize_field("parameterId", &ParameterId::PID_RELIABILITY.value)?;
        s.serialize_field::<u16>("parameterLength", &12)?;
        s.serialize_field("reliability", &self.qos.reliability())?;

        // destination_order
        s.serialize_field("parameterId", &ParameterId::PID_DESTINATION_ORDER.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("reliability", &self.qos.destination_order())?;

        // history
        s.serialize_field("parameterId", &ParameterId::PID_HISTORY.value)?;
        s.serialize_field::<u16>("parameterLength", &8)?;
        s.serialize_field("reliability", &self.qos.history())?;

        // resouce_limits
        s.serialize_field("parameterId", &ParameterId::PID_RESOURCE_LIMITS.value)?;
        s.serialize_field::<u16>("parameterLength", &12)?;
        s.serialize_field("reliability", &self.qos.resource_limits())?;

        // transport_priority
        s.serialize_field("parameterId", &ParameterId::PID_TRANSPORT_PRIO.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("reliability", &self.qos.transport_priority())?;

        // lifespan
        s.serialize_field("parameterId", &ParameterId::PID_LIFESPAN.value)?;
        s.serialize_field::<u16>("parameterLength", &8)?;
        s.serialize_field("lifespan", &self.qos.lifespan())?;

        // user_data
        s.serialize_field("parameterId", &ParameterId::PID_USER_DATA.value)?;
        s.serialize_field::<u16>(
            "parameterLength",
            &(4 + self.qos.user_data().value.len() as u16),
        )?;
        s.serialize_field("user_data", &self.qos.user_data())?;

        // ownership
        s.serialize_field("parameterId", &ParameterId::PID_OWNERSHIP.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("ownership", &self.qos.ownership())?;

        // ownership_strength
        s.serialize_field("parameterId", &ParameterId::PID_OWNERSHIP_STRENGTH.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("ownership_strength", &self.qos.ownership_strength())?;

        s.end()
    }
}
