use crate::dds::{
    datareader::DataReader, participant::DomainParticipant, qos::QosPolicies, topic::Topic,
};
use crate::rtps::{cache::HistoryCache, reader::ReaderIngredients};
use crate::structure::{
    entity::RTPSEntity,
    entity_id::{EntityId, EntityKind},
    guid::GUID,
};
use mio_extras::channel as mio_channel;
use serde::Serialize;
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
    pub fn create_datareader<D: Serialize>(&self, qos: QosPolicies, topic: Topic) -> DataReader<D> {
        self.inner.create_datareader(qos, topic, self.clone())
    }
    pub fn create_datareader_with_entityid<D: Serialize>(
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

    pub fn create_datareader<D: Serialize>(
        &self,
        qos: QosPolicies,
        topic: Topic,
        subscriber: Subscriber,
    ) -> DataReader<D> {
        let history_cache = Arc::new(RwLock::new(HistoryCache::new()));
        let reader_ing = ReaderIngredients {
            guid: GUID::new(
                self.dp.guid_prefix(),
                EntityId::new_with_entity_kind(
                    self.dp.gen_entity_key(),
                    EntityKind::READER_WITH_KEY_USER_DEFIND,
                ),
            ),
            rhc: history_cache.clone(),
        };
        self.add_reader_sender.send(reader_ing).unwrap();
        DataReader::<D>::new(qos, topic, subscriber, history_cache)
    }

    pub fn create_datareader_with_entityid<D: Serialize>(
        &self,
        qos: QosPolicies,
        topic: Topic,
        subscriber: Subscriber,
        entity_id: EntityId,
    ) -> DataReader<D> {
        let history_cache = Arc::new(RwLock::new(HistoryCache::new()));
        let reader_ing = ReaderIngredients {
            guid: GUID::new(self.dp.guid_prefix(), entity_id),
            rhc: history_cache.clone(),
        };
        self.add_reader_sender.send(reader_ing).unwrap();
        DataReader::<D>::new(qos, topic, subscriber, history_cache)
    }
}
