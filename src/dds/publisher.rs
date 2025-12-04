use crate::dds::{
    datawriter::DataWriter,
    participant::DomainParticipant,
    qos::policy::*,
    qos::{DataWriterQos, DataWriterQosBuilder, DataWriterQosPolicies, PublisherQosPolicies},
    topic::Topic,
};
use crate::discovery::ParticipantMessageCmd;
use crate::message::submessage::element::Locator;
use crate::network::net_util::{usertraffic_multicast_port, usertraffic_unicast_port};
use crate::rtps::cache::{HistoryCache, HistoryCacheType};
use crate::rtps::writer::{DataWriterStatusChanged, WriterCmd, WriterIngredients};
use crate::structure::{Duration, EntityId, EntityKind, RTPSEntity, TopicKind, GUID};
use crate::DdsData;
use alloc::sync::Arc;
use awkernel_sync::rwlock::RwLock;
use mio_extras::channel as mio_channel;

/// DDS Publisher
///
/// factory of DataWriter
#[derive(Clone)]
pub struct Publisher {
    inner: Arc<RwLock<InnerPublisher>>,
}

#[allow(dead_code)]
struct InnerPublisher {
    guid: GUID,
    // rtps 2.3 spec 8.2.4.4
    // The DDS Specification defines Publisher and Subscriber entities.
    // These two entities have GUIDs that are defined exactly
    // as described for Endpoints in clause 8.2.4.3 above.
    qos: PublisherQosPolicies,
    default_dw_qos: DataWriterQosPolicies,
    dp: DomainParticipant,
    create_writer_sender: mio_channel::SyncSender<WriterIngredients>,
    participant_msg_cmd_sender: mio_channel::SyncSender<ParticipantMessageCmd>,
}

impl Publisher {
    pub fn new(
        guid: GUID,
        qos: PublisherQosPolicies,
        dp: DomainParticipant,
        create_writer_sender: mio_channel::SyncSender<WriterIngredients>,
        participant_msg_cmd_sender: mio_channel::SyncSender<ParticipantMessageCmd>,
    ) -> Self {
        let default_dw_qos = DataWriterQosBuilder::new().build();
        Self {
            inner: Arc::new(RwLock::new(InnerPublisher::new(
                guid,
                qos,
                default_dw_qos,
                dp,
                create_writer_sender,
                participant_msg_cmd_sender,
            ))),
        }
    }

    /// Note that if you pass `DataWriterQos::Policies(qos)` when creating a DataWriter,
    /// the resulting QoS will not be exactly the same as the provided `qos`.
    ///
    /// Instead, the DataWriter QoS is constructed by combining:
    /// 1) the QoS from the associated Topic (`topic_qos`),
    /// 2) the Publisher's default DataWriter QoS (`publisher.get_default_datawriter_qos()`), and
    /// 3) the user-supplied `qos`.
    ///
    /// The pseudo-code below illustrates the combination process:
    /// ```ignore
    /// impl DataWriterQosPolicies {
    ///     fn combine(&mut self, other: Self) {
    ///         // qos_policy: (QosPolicy, bool)
    ///         // The bool flag indicates whether the policy was explicitly set by the user.
    ///
    ///         // For each QoS policy in Self:
    ///         for qos_policy in Self {
    ///             // If the policies differ, select the one explicitly specified by the user.
    ///             if self.qos_policy.0 != qos_policy.0 && qos_policy.1 {
    ///                     self.qos_policy = qos_policy;
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// let mut dw_qos = topic_qos.to_datawriter_qos();
    /// dw_qos.combine(publisher.get_default_datawriter_qos());
    /// dw_qos.combine(qos);
    /// dw_qos
    /// ```
    ///
    /// If you set `DataWriterQos::Default` as the QoS when creating a DataWriter,
    /// it simply uses the Publisher's default DataWriter
    /// QoS (publisher.get_default_datawriter_qos()). Therefore,
    /// ```ignore
    /// publisher.create_datawriter::<Hoge>(DataWriterQos::Default, &topic)
    /// ```
    /// is may **not** equivalent to:
    /// ```ignore
    /// publisher.create_datawriter::<Hoge>(publisher.get_default_datawriter_qos(), &topic)
    /// ```
    pub fn create_datawriter<D: serde::Serialize + DdsData>(
        &self,
        qos: DataWriterQos,
        topic: Topic,
    ) -> DataWriter<D> {
        self.inner
            .read()
            .create_datawriter(qos, topic, self.clone())
    }

    /// See [`Self::create_datawriter`] for a note of qos.
    pub fn create_datawriter_with_entityid<D: serde::Serialize + DdsData>(
        &self,
        qos: DataWriterQos,
        topic: Topic,
        entity_id: EntityId,
    ) -> DataWriter<D> {
        self.inner
            .read()
            .create_datawriter_with_entityid(qos, topic, self.clone(), entity_id)
    }

    pub fn get_qos(&self) -> PublisherQosPolicies {
        self.inner.read().get_qos()
    }
    pub fn set_qos(&mut self, qos: PublisherQosPolicies) {
        self.inner.write().set_qos(qos);
    }

    pub fn domain_participant(&self) -> DomainParticipant {
        self.inner.read().dp.clone()
    }
    pub fn get_default_datawriter_qos(&self) -> DataWriterQosPolicies {
        self.inner.read().default_dw_qos.clone()
    }
    pub fn set_default_datawriter_qos(&mut self, qos: DataWriterQosPolicies) {
        self.inner.write().default_dw_qos = qos;
    }
}

#[allow(dead_code)]
impl InnerPublisher {
    pub fn new(
        guid: GUID,
        qos: PublisherQosPolicies,
        default_dw_qos: DataWriterQosPolicies,
        dp: DomainParticipant,
        create_writer_sender: mio_channel::SyncSender<WriterIngredients>,
        participant_msg_cmd_sender: mio_channel::SyncSender<ParticipantMessageCmd>,
    ) -> Self {
        Self {
            guid,
            qos,
            default_dw_qos,
            dp,
            create_writer_sender,
            participant_msg_cmd_sender,
        }
    }

    pub fn get_qos(&self) -> PublisherQosPolicies {
        self.qos.clone()
    }

    pub fn set_qos(&mut self, qos: PublisherQosPolicies) {
        self.qos = qos;
    }

    pub fn create_datawriter<D: serde::Serialize + DdsData>(
        &self,
        qos: DataWriterQos,
        topic: Topic,
        outter: Publisher,
    ) -> DataWriter<D> {
        let entity_kind = match topic.kind() {
            TopicKind::WithKey => EntityKind::WRITER_WITH_KEY_USER_DEFIND,
            TopicKind::NoKey => EntityKind::WRITER_NO_KEY_USER_DEFIND,
        };
        let entity_id = EntityId::new_with_entity_kind(self.dp.gen_entity_key(), entity_kind);
        self.create_datawriter_with_entityid(qos, topic, outter, entity_id)
    }

    pub fn create_datawriter_with_entityid<D: serde::Serialize + DdsData>(
        &self,
        qos: DataWriterQos,
        topic: Topic,
        outter: Publisher,
        entity_id: EntityId,
    ) -> DataWriter<D> {
        let dw_qos = match qos {
            // DDS 1.4 spec, 2.2.2.4.1.5 create_datawriter
            // > The special value DATAWRITER_QOS_DEFAULT can be used to indicate that the DataWriter should be created with the
            // default DataWriter QoS set in the factory. The use of this value is equivalent to the application obtaining the default
            // DataWriter QoS by means of the operation get_default_datawriter_qos (2.2.2.4.1.15) and using the resulting QoS to create
            // the DataWriter.
            DataWriterQos::Default => self.default_dw_qos.clone(),
            // DDS 1.4 spec, 2.2.2.4.1.5 create_datawriter
            // > Note that a common application pattern to construct the QoS for the DataWriter is to:
            // > + Retrieve the QoS policies on the associated Topic by means of the get_qos operation on the Topic.
            // > + Retrieve the default DataWriter qos by means of the get_default_datawriter_qos operation on the Publisher.
            // > + Combine those two QoS policies and selectively modify policies as desired.
            // > + Use the resulting QoS policies to construct the DataWriter.
            DataWriterQos::Policies(q) => {
                let mut dw_qos = topic.my_qos_policies().to_datawriter_qos();
                dw_qos.combine(self.default_dw_qos.clone());
                dw_qos.combine(*q);
                dw_qos
            }
        };
        let (writer_state_notifier, writer_state_receiver) =
            mio_channel::channel::<DataWriterStatusChanged>();
        let (writer_command_sender, writer_command_receiver) =
            mio_channel::sync_channel::<WriterCmd>(4);
        let history_cache = Arc::new(RwLock::new(HistoryCache::new(HistoryCacheType::Writer)));
        let reliability_level = dw_qos.reliability().kind;
        let heartbeat_period = match reliability_level {
            ReliabilityQosKind::Reliable => Duration::from_secs(2),
            ReliabilityQosKind::BestEffort => Duration::ZERO,
        };
        let domain_id = self.dp.domain_id();
        let participant_id = self.dp.participant_id();
        let nics = self.dp.get_network_interfaces();
        let unicast_locator_list = Locator::new_list_from_multi_ipv4(
            usertraffic_unicast_port(domain_id, participant_id) as u32,
            nics,
        );
        let guid = GUID::new(self.dp.guid_prefix(), entity_id);
        let writer_ing = WriterIngredients {
            guid,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list: vec![Locator::new_from_ipv4(
                usertraffic_multicast_port(domain_id) as u32,
                [239, 255, 0, 1],
            )],
            push_mode: true,
            heartbeat_period,
            nack_response_delay: Duration::from_millis(200),
            nack_suppression_duration: Duration::ZERO,
            data_max_size_serialized: 0,
            whc: history_cache.clone(),
            topic: topic.clone(),
            qos: dw_qos.clone(),
            writer_command_receiver,
            writer_state_notifier,
            participant_msg_cmd_sender: self.participant_msg_cmd_sender.clone(),
        };
        self.create_writer_sender
            .send(writer_ing)
            .expect("couldn't send channel 'create_writer_sender'");
        DataWriter::<D>::new(
            writer_command_sender,
            guid,
            dw_qos,
            topic,
            outter,
            history_cache,
            writer_state_receiver,
        )
    }
    pub fn get_participant(&self) -> DomainParticipant {
        self.dp.clone()
    }
    pub fn get_default_datawriter_qos(&self) -> DataWriterQosPolicies {
        self.default_dw_qos.clone()
    }
    pub fn set_default_datawriter_qos(&mut self, qos: DataWriterQosPolicies) {
        self.default_dw_qos = qos;
    }
    pub fn suspend_publications(&self) {}
    pub fn resume_publications(&self) {}
    pub fn begin_coherent_change(&self) {}
    pub fn end_coherent_changes(&self) {}
    pub fn wait_for_acknowledgments(&self) {}
}
