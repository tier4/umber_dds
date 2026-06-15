use crate::dds::qos::{policy::Durability, DataReaderQosPolicies, DataWriterQosPolicies};
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
use log::{debug, warn};
use speedy::{Context, Writable, Writer};

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
        let durability = qos.durability();
        {
            let hc = history_cache.read();
            for k in hc.changes.keys() {
                let latest = hc.ts2key[hc.ts2key.len() - 1];
                let is_relevant = {
                    match durability {
                        Durability::Volatile => false,
                        Durability::TransientLocal => {
                            if remote_reader_guid.entity_id.is_builtin() {
                                true
                            } else {
                                *k == latest
                            }
                        }
                    }
                };
                cache_state.insert(
                    k.seq_num,
                    ChangeForReader::new(k.seq_num, status_kind, is_relevant),
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
        debug!(
            "ReaderProxy.update_cache_state({:?}, {}, {:?})\n\tReader: {}",
            seq_num, is_relevant, state, self.remote_reader_guid
        );
    }

    pub fn acked_changes_set(&mut self, commited_seq_num: SequenceNumber) {
        let mut to_update = Vec::new();
        for (seq_num, cfr) in &self.cache_state {
            if *seq_num <= commited_seq_num {
                to_update.push((*seq_num, cfr.is_relevant));
            }
        }
        for (seq_num, is_relevant) in to_update {
            self.update_cache_state(
                seq_num,
                is_relevant,
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
    pub fn is_acked(&self, seq_num: SequenceNumber) -> bool {
        if let Some(change) = self.cache_state.get(&seq_num) {
            if change.is_relevant {
                change.status == ChangeForReaderStatusKind::Acknowledged
            } else {
                true
            }
        } else {
            true
        }
    }
    pub fn remove_cache_state(&mut self, seq_num: &SequenceNumber) {
        debug!(
            "ReaderProxy.remove_cache_state({:?})\n\tReader: {}",
            seq_num, self.remote_reader_guid
        );
        if self.cache_state.remove(seq_num).is_none() {
            warn!(
                "ReaderProxy requested to non-existent cache with {:?}\n\tReader: {}",
                seq_num, self.remote_reader_guid
            );
        }
    }

    #[allow(dead_code)]
    pub fn unacked_changes(&self) -> Vec<ChangeForReader> {
        let mut unacked_changes = Vec::new();
        for change_for_reader in self.cache_state.values() {
            if let ChangeForReaderStatusKind::Unacknowledged = change_for_reader.status {
                unacked_changes.push(change_for_reader.clone())
            }
        }
        unacked_changes
    }
}

impl<C: Context> Writable<C> for ReaderProxy {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        // remote_reader_guid
        writer.write_u16(ParameterId::PID_ENDPOINT_GUID.value)?;
        writer.write_u16(16)?;
        writer.write_value(&self.remote_reader_guid)?;

        // expects_inline_qos
        writer.write_u16(ParameterId::PID_EXPECTS_INLINE_QOS.value)?;
        writer.write_u16(4)?;
        // writer.write_value(&self.expects_inline_qos)?;
        if self.expects_inline_qos {
            writer.write_u8(1)?;
        } else {
            writer.write_u8(0)?;
        }
        // padding alignment to 4 bytes for bool + u16
        writer.write_bytes(&[0; 3])?;

        // unicast_locator_list
        for unicast_locator in &self.unicast_locator_list {
            writer.write_u16(ParameterId::PID_UNICAST_LOCATOR.value)?;
            writer.write_u16(24)?;
            writer.write_value(unicast_locator)?;
        }

        // multicast_locator_list
        for multicast_locator in &self.multicast_locator_list {
            writer.write_u16(ParameterId::PID_MULTICAST_LOCATOR.value)?;
            writer.write_u16(24)?;
            writer.write_value(multicast_locator)?;
        }

        // durability
        writer.write_u16(ParameterId::PID_DURABILITY.value)?;
        writer.write_u16(4)?;
        writer.write_value(&self.qos.durability())?;

        // deadline
        writer.write_u16(ParameterId::PID_DEADLINE.value)?;
        writer.write_u16(8)?;
        writer.write_value(&self.qos.deadline())?;

        // latency_budget
        writer.write_u16(ParameterId::PID_LATENCY_BUDGET.value)?;
        writer.write_u16(8)?;
        writer.write_value(&self.qos.latency_budget())?;

        // liveliness
        writer.write_u16(ParameterId::PID_LIVELINESS.value)?;
        writer.write_u16(12)?;
        writer.write_value(&self.qos.liveliness())?;

        // reliability
        writer.write_u16(ParameterId::PID_RELIABILITY.value)?;
        writer.write_u16(12)?;
        writer.write_value(&self.qos.reliability())?;

        // destination_order
        writer.write_u16(ParameterId::PID_DESTINATION_ORDER.value)?;
        writer.write_u16(4)?;
        writer.write_value(&self.qos.destination_order())?;

        // history
        writer.write_u16(ParameterId::PID_HISTORY.value)?;
        writer.write_u16(8)?;
        writer.write_value(&self.qos.history())?;

        // resouce_limits
        writer.write_u16(ParameterId::PID_RESOURCE_LIMITS.value)?;
        writer.write_u16(12)?;
        writer.write_value(&self.qos.resource_limits())?;

        // user_data
        writer.write_u16(ParameterId::PID_USER_DATA.value)?;
        writer.write_u16(4 + self.qos.user_data().value.len() as u16)?;
        writer.write_value(&self.qos.user_data())?;

        // ownership
        writer.write_u16(ParameterId::PID_OWNERSHIP.value)?;
        writer.write_u16(4)?;
        writer.write_value(&self.qos.ownership())?;

        // time_based_filter
        writer.write_u16(ParameterId::PID_TIME_BASED_FILTER.value)?;
        writer.write_u16(8)?;
        writer.write_value(&self.qos.time_based_filter())?;

        Ok(())
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
        debug!(
            "WriterProxy.update_cache_state({:?}, {}, {:?})\n\tWriter: {}",
            seq_num, is_relevant, state, self.remote_writer_guid
        );
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
                ChangeFromWriterStatusKind::_Uuknown | ChangeFromWriterStatusKind::Missing
                    if *sn < first_available_seq_num =>
                {
                    cfw.status = ChangeFromWriterStatusKind::Lost;
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
                if let ChangeFromWriterStatusKind::_Uuknown = cfw.status {
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
    pub fn received_change_set(&mut self, seq_num: SequenceNumber) {
        if let Some(cfw) = self.cache_state.get_mut(&seq_num) {
            cfw.status = ChangeFromWriterStatusKind::Received;
        } else {
            self.update_cache_state(seq_num, true, ChangeFromWriterStatusKind::Received);
        }
    }
    pub fn remove_cache_state(&mut self, seq_num: &SequenceNumber) {
        // self.cache_state.remove(seq_num);
        debug!(
            "WriterProxy.remove_cache_state({:?})\n\tWriter: {}",
            seq_num, self.remote_writer_guid
        );
        if self.cache_state.remove(seq_num).is_none() {
            warn!(
                "WriterProxy requested to non-existent cache with {:?}\n\tWriter: {}",
                seq_num, self.remote_writer_guid
            );
        }
    }
}

impl<C: Context> Writable<C> for WriterProxy {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        // remote_writer_guid
        writer.write_u16(ParameterId::PID_ENDPOINT_GUID.value)?;
        writer.write_u16(16)?;
        writer.write_value(&self.remote_writer_guid)?;

        // unicast_locator_list
        for unicast_locator in &self.unicast_locator_list {
            writer.write_u16(ParameterId::PID_UNICAST_LOCATOR.value)?;
            writer.write_u16(24)?;
            writer.write_value(unicast_locator)?;
        }

        // multicast_locator_list
        for multicast_locator in &self.multicast_locator_list {
            writer.write_u16(ParameterId::PID_MULTICAST_LOCATOR.value)?;
            writer.write_u16(24)?;
            writer.write_value(multicast_locator)?;
        }

        // data_max_size_serialized
        writer.write_u16(ParameterId::PID_TYPE_MAX_SIZE_SERIALIZED.value)?;
        writer.write_u16(4)?;
        writer.write_i32(self.data_max_size_serialized)?;

        // durability
        writer.write_u16(ParameterId::PID_DURABILITY.value)?;
        writer.write_u16(4)?;
        writer.write_value(&self.qos.durability())?;

        // durability_service
        writer.write_u16(ParameterId::PID_DURABILITY_SERVICE.value)?;
        writer.write_u16(28)?;
        writer.write_value(&self.qos.durability_service())?;

        // deadline
        writer.write_u16(ParameterId::PID_DEADLINE.value)?;
        writer.write_u16(8)?;
        writer.write_value(&self.qos.deadline())?;

        // latency_budget
        writer.write_u16(ParameterId::PID_LATENCY_BUDGET.value)?;
        writer.write_u16(8)?;
        writer.write_value(&self.qos.latency_budget())?;

        // liveliness
        writer.write_u16(ParameterId::PID_LIVELINESS.value)?;
        writer.write_u16(12)?;
        writer.write_value(&self.qos.liveliness())?;

        // reliability
        writer.write_u16(ParameterId::PID_RELIABILITY.value)?;
        writer.write_u16(12)?;
        writer.write_value(&self.qos.reliability())?;

        // destination_order
        writer.write_u16(ParameterId::PID_DESTINATION_ORDER.value)?;
        writer.write_u16(4)?;
        writer.write_value(&self.qos.destination_order())?;

        // history
        writer.write_u16(ParameterId::PID_HISTORY.value)?;
        writer.write_u16(8)?;
        writer.write_value(&self.qos.history())?;

        // resouce_limits
        writer.write_u16(ParameterId::PID_RESOURCE_LIMITS.value)?;
        writer.write_u16(12)?;
        writer.write_value(&self.qos.resource_limits())?;

        // transport_priority
        writer.write_u16(ParameterId::PID_TRANSPORT_PRIORITY.value)?;
        writer.write_u16(4)?;
        writer.write_value(&self.qos.transport_priority())?;

        // lifespan
        writer.write_u16(ParameterId::PID_LIFESPAN.value)?;
        writer.write_u16(8)?;
        writer.write_value(&self.qos.lifespan())?;

        // user_data
        writer.write_u16(ParameterId::PID_USER_DATA.value)?;
        writer.write_u16(4 + self.qos.user_data().value.len() as u16)?;
        writer.write_value(&self.qos.user_data())?;

        // ownership
        writer.write_u16(ParameterId::PID_OWNERSHIP.value)?;
        writer.write_u16(4)?;
        writer.write_value(&self.qos.ownership())?;

        // ownership_strength
        writer.write_u16(ParameterId::PID_OWNERSHIP_STRENGTH.value)?;
        writer.write_u16(4)?;
        writer.write_value(&self.qos.ownership_strength())?;

        Ok(())
    }
}
