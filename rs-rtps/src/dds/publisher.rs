use crate::dds::{
    datawriter::DataWriter, participant::DomainParticipant, qos::policy::*, qos::QosPolicies,
    topic::Topic,
};
use crate::message::submessage::element::Locator;
use crate::network::net_util::{usertraffic_multicast_port, usertraffic_unicast_port};
use crate::rtps::writer::{WriterCmd, WriterIngredients};
use crate::structure::{
    duration::Duration,
    entity::RTPSEntity,
    entity_id::{EntityId, EntityKind},
    guid::GUID,
};
use mio_extras::channel as mio_channel;
use std::sync::Arc;

#[derive(Clone)]
pub struct Publisher {
    inner: Arc<InnerPublisher>,
}

struct InnerPublisher {
    guid: GUID,
    // rtps 2.3 spec 8.2.4.4
    // The DDS Specification defines Publisher and Subscriber entities.
    // These two entities have GUIDs that are defined exactly
    // as described for Endpoints in clause 8.2.4.3 above.
    qos: QosPolicies,
    default_dw_qos: QosPolicies,
    dp: DomainParticipant,
    add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
}

impl Publisher {
    pub fn new(
        guid: GUID,
        qos: QosPolicies,
        default_dw_qos: QosPolicies,
        dp: DomainParticipant,
        add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
    ) -> Self {
        Self {
            inner: Arc::new(InnerPublisher::new(
                guid,
                qos,
                default_dw_qos,
                dp,
                add_writer_sender,
            )),
        }
    }

    pub fn create_datawriter<D: serde::Serialize>(
        &self,
        qos: QosPolicies,
        topic: Topic,
    ) -> DataWriter<D> {
        self.inner.create_datawriter(qos, topic, self.clone())
    }

    pub fn create_datawriter_with_entityid<D: serde::Serialize>(
        &self,
        qos: QosPolicies,
        topic: Topic,
        entity_id: EntityId,
    ) -> DataWriter<D> {
        self.inner
            .create_datawriter_with_entityid(qos, topic, self.clone(), entity_id)
    }

    pub fn domain_participant(&self) -> DomainParticipant {
        self.inner.dp.clone()
    }
}

impl InnerPublisher {
    pub fn new(
        guid: GUID,
        qos: QosPolicies,
        default_dw_qos: QosPolicies,
        dp: DomainParticipant,
        add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
    ) -> Self {
        Self {
            guid,
            qos,
            default_dw_qos,
            dp,
            add_writer_sender,
        }
    }

    /// Allows access to the values of the QoS.
    pub fn get_qos(&self) -> &QosPolicies {
        &self.qos
    }
    pub fn set_qos(&mut self, qos: QosPolicies) {
        self.qos = qos;
    }
    pub fn create_datawriter<D: serde::Serialize>(
        &self,
        qos: QosPolicies,
        topic: Topic,
        outter: Publisher,
    ) -> DataWriter<D> {
        let (writer_command_sender, writer_command_receiver) =
            mio_channel::sync_channel::<WriterCmd>(4);
        let reliability_level = if let Some(reliability) = qos.reliability {
            reliability.kind
        } else {
            ReliabilityQosKind::BestEffort // If qos don't specify reliability_level, the
                                           // reliability_level of Writer is set to BestEffort
        };
        let writer_ing = WriterIngredients {
            guid: GUID::new(
                self.dp.guid_prefix(),
                EntityId::new_with_entity_kind(
                    self.dp.gen_entity_key(),
                    EntityKind::WRITER_WITH_KEY_USER_DEFIND,
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
            push_mode: true,
            heartbeat_period: Duration::new(5, 0),
            nack_response_delay: Duration::new(0, 200 * 1000 * 1000),
            nack_suppression_duration: Duration::ZERO,
            data_max_size_serialized: 0,
            topic: topic.clone(),
            writer_command_receiver,
        };
        self.add_writer_sender.send(writer_ing).unwrap();
        DataWriter::<D>::new(writer_command_sender, qos, topic, outter)
    }
    pub fn create_datawriter_with_entityid<D: serde::Serialize>(
        &self,
        qos: QosPolicies,
        topic: Topic,
        outter: Publisher,
        entity_id: EntityId,
    ) -> DataWriter<D> {
        let (writer_command_sender, writer_command_receiver) =
            mio_channel::sync_channel::<WriterCmd>(4);
        let mut heartbeat_period = Duration::ZERO;
        let reliability_level = if let Some(reliability) = qos.reliability {
            heartbeat_period = Duration::new(5, 0);
            reliability.kind
        } else {
            ReliabilityQosKind::BestEffort // If qos don't specify reliability_level, the
                                           // reliability_level of Writer is set to BestEffort
        };
        let writer_ing = WriterIngredients {
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
            push_mode: true,
            heartbeat_period,
            nack_response_delay: Duration::new(0, 200 * 1000 * 1000),
            nack_suppression_duration: Duration::ZERO,
            data_max_size_serialized: 0,
            topic: topic.clone(),
            writer_command_receiver,
        };
        self.add_writer_sender.send(writer_ing).unwrap();
        DataWriter::<D>::new(writer_command_sender, qos, topic, outter)
    }
    pub fn get_participant(&self) -> DomainParticipant {
        self.dp.clone()
    }
    pub fn get_default_datawriter_qos(&self) -> QosPolicies {
        self.default_dw_qos
    }
    pub fn set_default_datawriter_qos(&mut self, qos: QosPolicies) {
        self.default_dw_qos = qos;
    }
    pub fn suspend_publications(&self) {}
    pub fn resume_publications(&self) {}
    pub fn begin_coherent_change(&self) {}
    pub fn end_coherent_changes(&self) {}
    pub fn wait_for_acknowledgments(&self) {}
}
