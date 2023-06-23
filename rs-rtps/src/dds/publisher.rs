use crate::dds::{
    datawriter::DataWriter, participant::DomainParticipant, qos::QosPolicies, topic::Topic,
};
use crate::rtps::writer::*;
use crate::structure::entityId::EntityId;
use mio_extras::channel as mio_channel;
use std::sync::Arc;

pub struct Publisher {
    inner: Arc<InnerPublisher>,
}

pub struct InnerPublisher {
    id: EntityId,
    qos: QosPolicies,
    default_dw_qos: QosPolicies,
    dp: DomainParticipant,
    add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
}

impl Publisher {
    pub fn new(
        qos: QosPolicies,
        default_dw_qos: QosPolicies,
        dp: DomainParticipant,
        add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
    ) -> Self {
        Self {
            inner: Arc::new(InnerPublisher {
                id: EntityId::MAX,
                // EntityId mut uniq, but we do not show it to anyone.
                qos,
                default_dw_qos,
                dp,
                add_writer_sender,
            }),
        }
    }

    pub fn create_datawriter<D: serde::Serialize>(
        &self,
        qos: QosPolicies,
        topic: Topic,
    ) -> DataWriter<D> {
        self.inner.create_datawriter(qos, topic, *self.clone())
    }

    pub fn domain_participant(&self) -> DomainParticipant {
        self.inner.dp
    }
}

impl InnerPublisher {
    pub fn new(
        qos: QosPolicies,
        default_dw_qos: QosPolicies,
        dp: DomainParticipant,
        add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
    ) -> Self {
        Self {
            id: EntityId::MAX,
            // EntityId mut uniq, but we do not show it to anyone.
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
        let entity_id = EntityId::WRITER_WITH_KEY_USER_DEFIND;
        let guid = self.dp.guid().new_from_id(entity_id);
        DataWriter::<D>::new(self.add_writer_sender, qos, topic, outter)
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
