use crate::dds::{
    datareader::DataReader, participant::DomainParticipant, qos::policy::*, qos::QosPolicies,
    topic::Topic,
};
use crate::message::submessage::element::Locator;
use crate::network::net_util::{usertraffic_multicast_port, usertraffic_unicast_port};
use crate::rtps::{cache::HistoryCache, reader::ReaderIngredients};
use crate::structure::{
    duration::Duration,
    entity::RTPSEntity,
    entity_id::{EntityId, EntityKind},
    guid::GUID,
};
use mio_extras::channel as mio_channel;
use serde::Deserialize;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Subscriber {
    inner: Arc<InnerSubscriber>,
}

impl Subscriber {
    pub fn new(
        guid: GUID,
        qos: QosPolicies,
        dp: DomainParticipant,
        add_reader_sender: mio_channel::SyncSender<ReaderIngredients>,
    ) -> Self {
        Self {
            inner: Arc::new(InnerSubscriber::new(guid, qos, dp, add_reader_sender)),
        }
    }
    pub fn create_datareader<D: for<'de> Deserialize<'de>>(
        &self,
        qos: QosPolicies,
        topic: Topic,
    ) -> DataReader<D> {
        self.inner.create_datareader(qos, topic, self.clone())
    }
    pub fn create_datareader_with_entityid<D: for<'de> Deserialize<'de>>(
        &self,
        qos: QosPolicies,
        topic: Topic,
        entity_id: EntityId,
    ) -> DataReader<D> {
        self.inner
            .create_datareader_with_entityid(qos, topic, self.clone(), entity_id)
    }
}

struct InnerSubscriber {
    guid: GUID,
    // rtps 2.3 spec 8.2.4.4
    // The DDS Specification defines Publisher and Subscriber entities.
    // These two entities have GUIDs that are defined exactly
    // as described for Endpoints in clause 8.2.4.3 above.
    qos: QosPolicies,
    dp: DomainParticipant,
    add_reader_sender: mio_channel::SyncSender<ReaderIngredients>,
}

impl InnerSubscriber {
    pub fn new(
        guid: GUID,
        qos: QosPolicies,
        dp: DomainParticipant,
        add_reader_sender: mio_channel::SyncSender<ReaderIngredients>,
    ) -> Self {
        Self {
            guid,
            qos,
            dp,
            add_reader_sender,
        }
    }

    pub fn get_qos(&self) -> QosPolicies {
        self.qos
    }

    pub fn set_qos(&mut self, qos: QosPolicies) {
        self.qos = qos
    }

    pub fn create_datareader<D: for<'de> Deserialize<'de>>(
        &self,
        qos: QosPolicies,
        topic: Topic,
        subscriber: Subscriber,
    ) -> DataReader<D> {
        let (reader_ready_notifier, reader_ready_receiver) = mio_channel::channel::<()>();
        let history_cache = Arc::new(RwLock::new(HistoryCache::new()));
        let reliability_level = if let Some(reliability) = qos.reliability {
            reliability.kind
        } else {
            ReliabilityQosKind::BestEffort
        };
        let reader_ing = ReaderIngredients {
            guid: GUID::new(
                self.dp.guid_prefix(),
                EntityId::new_with_entity_kind(
                    self.dp.gen_entity_key(),
                    EntityKind::READER_WITH_KEY_USER_DEFIND,
                ),
            ),
            topic_kind: topic.kind(),
            reliability_level,
            unicast_locator_list: Locator::new_list_from_self_ipv4(usertraffic_unicast_port(
                self.dp.domain_id(),
                self.dp.participant_id(),
            ) as u32),
            multicast_locator_list: vec![Locator::new_from_ipv4(
                usertraffic_multicast_port(self.dp.domain_id()) as u32,
                [239, 255, 0, 1],
            )],
            expectsinline_qos: false,
            heartbeat_response_delay: Duration::ZERO,
            rhc: history_cache.clone(),
            topic: topic.clone(),
            reader_ready_notifier,
        };
        self.add_reader_sender
            .send(reader_ing)
            .expect("couldn't send channel 'add_reader_sender'");
        DataReader::<D>::new(qos, topic, subscriber, history_cache, reader_ready_receiver)
    }

    pub fn create_datareader_with_entityid<D: for<'de> Deserialize<'de>>(
        &self,
        qos: QosPolicies,
        topic: Topic,
        subscriber: Subscriber,
        entity_id: EntityId,
    ) -> DataReader<D> {
        let (reader_ready_notifier, reader_ready_receiver) = mio_channel::channel::<()>();
        let history_cache = Arc::new(RwLock::new(HistoryCache::new()));
        let reliability_level = if let Some(reliability) = qos.reliability {
            reliability.kind
        } else {
            ReliabilityQosKind::BestEffort
        };
        let reader_ing = ReaderIngredients {
            guid: GUID::new(self.dp.guid_prefix(), entity_id),
            topic_kind: topic.kind(),
            reliability_level,
            unicast_locator_list: Locator::new_list_from_self_ipv4(usertraffic_unicast_port(
                self.dp.domain_id(),
                self.dp.participant_id(),
            ) as u32),
            multicast_locator_list: vec![Locator::new_from_ipv4(
                usertraffic_multicast_port(self.dp.domain_id()) as u32,
                [239, 255, 0, 1],
            )],
            expectsinline_qos: false,
            heartbeat_response_delay: Duration::ZERO,
            rhc: history_cache.clone(),
            topic: topic.clone(),
            reader_ready_notifier,
        };
        self.add_reader_sender
            .send(reader_ing)
            .expect("couldn't send add_reader_sender");
        DataReader::<D>::new(qos, topic, subscriber, history_cache, reader_ready_receiver)
    }
}
