use crate::dds::{participant::DomainParticipant, qos::QosPolicies};

pub struct Subscriber {
    qos: QosPolicies,
}

impl Subscriber {
    pub fn new(qos: QosPolicies) -> Self {
        Self { qos }
    }

    pub fn get_qos(&self) -> QosPolicies {
        self.qos
    }

    pub fn set_qos(&mut self, qos: QosPolicies) {
        self.qos = qos
    }

    pub fn create_datawriter() {}
}
