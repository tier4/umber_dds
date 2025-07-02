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
use crate::structure::{Duration, EntityId, EntityKind, RTPSEntity, TopicKind, GUID};
use crate::DdsData;
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

    /// Note that if you pass `DataReaderQos::Policies(qos)` when creating a DataReader,
    /// the resulting QoS will not be exactly the same as the provided `qos`.
    ///
    /// Instead, the DataReader QoS is constructed by combining:
    /// 1) the QoS from the associated Topic (`topic_qos`),
    /// 2) the Subscriber's default DataReader QoS (`subscriber.get_default_datareader_qos()`), and
    /// 3) the user-supplied `qos`.
    ///
    /// The pseudo-code below illustrates the combination process:
    /// ```ignore
    /// impl DataReaderQosPolicies {
    ///     fn combine(&mut self, other: Self) {
    ///         // For each QoS policy in Self:
    ///         for qos_policy in Self {
    ///             // If `other`'s policy is not default and differs from `self`'s policy,
    ///             // overwrite `self`'s policy.
    ///             if self.qos_policy != qos_policy && qos_policy != QoSPolicy::default() {
    ///                 self.qos_policy = qos_policy;
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut dr_qos = topic_qos.to_datareader_qos();
    /// dr_qos.combine(subscriber.get_default_datareader_qos());
    /// dr_qos.combine(qos);
    /// dr_qos
    /// ```
    ///
    /// If you set `DataReaderQos::Default` as the QoS when creating a DataReader,
    /// it simply uses the Subscriber's default DataReader QoS
    /// (subscriber.get_default_datareader_qos()). Therefore,
    /// ```ignore
    /// subscriber.create_datareader::<Hoge>(DataReaderQos::Default, &topic)
    /// ```
    /// is may **not** equivalent to:
    /// ```ignore
    /// subscriber.create_datareader::<Hoge>(subscriber.get_default_datareader_qos(), &topic)
    /// ```
    pub fn create_datareader<D: for<'de> Deserialize<'de> + DdsData>(
        &self,
        qos: DataReaderQos,
        topic: Topic,
    ) -> DataReader<D> {
        self.inner
            .read()
            .create_datareader(qos, topic, self.clone())
    }

    /// See [`Self::create_datareader`] for a note of qos.
    pub fn create_datareader_with_entityid<D: for<'de> Deserialize<'de> + DdsData>(
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

    pub fn create_datareader<D: for<'de> Deserialize<'de> + DdsData>(
        &self,
        qos: DataReaderQos,
        topic: Topic,
        subscriber: Subscriber,
    ) -> DataReader<D> {
        let entity_kind = match topic.kind() {
            TopicKind::WithKey => EntityKind::READER_WITH_KEY_USER_DEFIND,
            TopicKind::NoKey => EntityKind::READER_NO_KEY_USER_DEFIND,
        };
        let entity_id = EntityId::new_with_entity_kind(self.dp.gen_entity_key(), entity_kind);
        self.create_datareader_with_entityid(qos, topic, subscriber, entity_id)
    }

    pub fn create_datareader_with_entityid<D: for<'de> Deserialize<'de> + DdsData>(
        &self,
        qos: DataReaderQos,
        topic: Topic,
        subscriber: Subscriber,
        entity_id: EntityId,
    ) -> DataReader<D> {
        let dr_qos = match qos {
            // DDS 1.4 spec, 2.2.2.5.2.5 create_datareader
            // > The special value DATAREADER_QOS_DEFAULT can be used to indicate that the DataReader should be created with the
            // > default DataReader QoS set in the factory. The use of this value is equivalent to the application obtaining the default
            // > DataReader QoS by means of the operation get_default_datareader_qos (2.2.2.4.1.15) and using the resulting QoS to create
            // > the DataReader.
            DataReaderQos::Default => self.default_dr_qos.clone(),
            // DDS 1.4 spec, 2.2.2.5.2.5 create_datareader
            // > Note that a common application pattern to construct the QoS for the DataReader is to:
            // > + Retrieve the QoS policies on the associated Topic by means of the get_qos operation on the Topic.
            // > + Retrieve the default DataReader qos by means of the get_default_datareader_qos operation on the Subscriber.
            // > + Combine those two QoS policies and selectively modify policies as desired.
            // > + Use the resulting QoS policies to construct the DataReader.
            DataReaderQos::Policies(q) => {
                let mut dr_qos = topic.my_qos_policies().to_datareader_qos();
                dr_qos.combine(self.default_dr_qos.clone());
                dr_qos.combine(q);
                dr_qos
            }
        };
        let (reader_state_notifier, reader_state_receiver) =
            mio_channel::channel::<DataReaderStatusChanged>();
        let history_cache = Arc::new(RwLock::new(HistoryCache::new()));
        let reliability_level = dr_qos.reliability().kind;
        let domain_id = self.dp.domain_id();
        let participant_id = self.dp.participant_id();
        let nics = self.dp.get_network_interfaces();
        let unicast_locator_list = Locator::new_list_from_multi_ipv4(
            usertraffic_unicast_port(domain_id, participant_id) as u32,
            nics,
        );
        let reader_ing = ReaderIngredients {
            guid: GUID::new(self.dp.guid_prefix(), entity_id),
            reliability_level,
            unicast_locator_list,
            multicast_locator_list: vec![Locator::new_from_ipv4(
                usertraffic_multicast_port(domain_id) as u32,
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
