use crate::dds::{participant::DomainParticipant, qos::QosPolicies};

pub struct Publisher {
    qos: QosPolicies,
    default_dw_qos: QosPolicies,
    dp: DomainParticipant,
}

impl Publisher {
    pub fn new(qos: QosPolicies, default_dw_qos: QosPolicies, dp: DomainParticipant) -> Self {
        Self {
            qos,
            default_dw_qos,
            dp,
        }
    }

    /// Allows access to the values of the QoS.
    pub fn get_qos(&self) -> &QosPolicies {
        &self.qos
    }
    pub fn set_qos(&mut self, qos: QosPolicies) {
        self.qos = qos;
    }
    pub fn create_datawriter() {}
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
