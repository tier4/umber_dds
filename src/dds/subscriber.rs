use crate::dds::{
    datareader::DataReader,
    participant::DomainParticipant,
    qos::{DataReaderQos, DataReaderQosBuilder, DataReaderQosPolicies, SubscriberQosPolicies},
    topic::Topic,
};
use crate::message::submessage::element::Locator;
use crate::network::net_util::{usertraffic_multicast_port, usertraffic_unicast_port};
use crate::rtps::{
    cache::HistoryCache,
    reader::{DataReaderStatusChanged, ReaderIngredients},
};
use crate::structure::{Duration, EntityId, EntityKind, RTPSEntity, GUID};
use alloc::sync::Arc;
use awkernel_sync::rwlock::RwLock;
use mio_extras::channel as mio_channel;
use serde::Deserialize;

/// DDS Subscriber
///
/// factory of DataReader
#[derive(Clone)]
pub struct Subscriber {
    inner: Arc<RwLock<InnerSubscriber>>,
}

impl Subscriber {
    pub fn new(
        guid: GUID,
        qos: SubscriberQosPolicies,
        dp: DomainParticipant,
        create_reader_sender: mio_channel::SyncSender<ReaderIngredients>,
    ) -> Self {
        let default_dr_qos = DataReaderQosBuilder::new().build();
        Self {
            inner: Arc::new(RwLock::new(InnerSubscriber::new(
                guid,
                qos,
                default_dr_qos,
                dp,
                create_reader_sender,
            ))),
        }
    }
    pub fn create_datareader<D: for<'de> Deserialize<'de>>(
        &self,
        qos: DataReaderQos,
        topic: Topic,
    ) -> DataReader<D> {
        self.inner
            .read()
            .create_datareader(qos, topic, self.clone())
    }
    pub fn create_datareader_with_entityid<D: for<'de> Deserialize<'de>>(
        &self,
        qos: DataReaderQos,
        topic: Topic,
        entity_id: EntityId,
    ) -> DataReader<D> {
        self.inner
            .read()
            .create_datareader_with_entityid(qos, topic, self.clone(), entity_id)
    }

    pub fn get_qos(&self) -> SubscriberQosPolicies {
        self.inner.read().get_qos()
    }
    pub fn set_qos(&mut self, qos: SubscriberQosPolicies) {
        self.inner.write().set_qos(qos)
    }
    pub fn get_default_datareader_qos(&self) -> DataReaderQosPolicies {
        self.inner.read().get_default_datareader_qos()
    }
    pub fn set_default_datareader_qos(&mut self, qos: DataReaderQosPolicies) {
        self.inner.write().set_default_datareader_qos(qos)
    }
}

#[allow(dead_code)]
struct InnerSubscriber {
    guid: GUID,
    // rtps 2.3 spec 8.2.4.4
    // The DDS Specification defines Publisher and Subscriber entities.
    // These two entities have GUIDs that are defined exactly
    // as described for Endpoints in clause 8.2.4.3 above.
    qos: SubscriberQosPolicies,
    default_dr_qos: DataReaderQosPolicies,
    dp: DomainParticipant,
    create_reader_sender: mio_channel::SyncSender<ReaderIngredients>,
}

impl InnerSubscriber {
    pub fn new(
        guid: GUID,
        qos: SubscriberQosPolicies,
        default_dr_qos: DataReaderQosPolicies,
        dp: DomainParticipant,
        create_reader_sender: mio_channel::SyncSender<ReaderIngredients>,
    ) -> Self {
        Self {
            guid,
            qos,
            default_dr_qos,
            dp,
            create_reader_sender,
        }
    }

    pub fn get_qos(&self) -> SubscriberQosPolicies {
        self.qos.clone()
    }

    pub fn set_qos(&mut self, qos: SubscriberQosPolicies) {
        self.qos = qos
    }

    pub fn create_datareader<D: for<'de> Deserialize<'de>>(
        &self,
        qos: DataReaderQos,
        topic: Topic,
        subscriber: Subscriber,
    ) -> DataReader<D> {
        let dr_qos = match qos {
            DataReaderQos::Default => self.default_dr_qos.clone(),
            DataReaderQos::Policies(q) => q,
        };
        let (reader_state_notifier, reader_state_receiver) =
            mio_channel::channel::<DataReaderStatusChanged>();
        let history_cache = Arc::new(RwLock::new(HistoryCache::new()));
        let reliability_level = dr_qos.reliability.kind;
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
            qos: dr_qos.clone(),
            reader_state_notifier,
        };
        self.create_reader_sender
            .send(reader_ing)
            .expect("couldn't send channel 'create_reader_sender'");
        DataReader::<D>::new(
            dr_qos,
            topic,
            subscriber,
            history_cache,
            reader_state_receiver,
        )
    }

    pub fn create_datareader_with_entityid<D: for<'de> Deserialize<'de>>(
        &self,
        qos: DataReaderQos,
        topic: Topic,
        subscriber: Subscriber,
        entity_id: EntityId,
    ) -> DataReader<D> {
        let dr_qos = match qos {
            DataReaderQos::Default => self.default_dr_qos.clone(),
            DataReaderQos::Policies(q) => q,
        };
        let (reader_state_notifier, reader_state_receiver) =
            mio_channel::channel::<DataReaderStatusChanged>();
        let history_cache = Arc::new(RwLock::new(HistoryCache::new()));
        let reliability_level = dr_qos.reliability.kind;
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
            qos: dr_qos.clone(),
            reader_state_notifier,
        };
        self.create_reader_sender
            .send(reader_ing)
            .expect("couldn't send create_reader_sender");
        DataReader::<D>::new(
            dr_qos,
            topic,
            subscriber,
            history_cache,
            reader_state_receiver,
        )
    }

    pub fn get_default_datareader_qos(&self) -> DataReaderQosPolicies {
        self.default_dr_qos.clone()
    }
    pub fn set_default_datareader_qos(&mut self, qos: DataReaderQosPolicies) {
        self.default_dr_qos = qos;
    }
}
