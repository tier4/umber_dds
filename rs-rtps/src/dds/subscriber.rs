use crate::dds::{participant::DomainParticipant, qos::QosPolicies};
use crate::structure::{entity::RTPSEntity, entity_id::EntityId, guid::GUID};

pub struct Subscriber {
    guid: GUID,
    // rtps 2.3 spec 8.2.4.4
    // The DDS Specification defines Publisher and Subscriber entities.
    // These two entities have GUIDs that are defined exactly
    // as described for Endpoints in clause 8.2.4.3 above.
    qos: QosPolicies,
}

impl Subscriber {
    pub fn new(guid: GUID, qos: QosPolicies) -> Self {
        Self { guid, qos }
    }

    pub fn get_qos(&self) -> QosPolicies {
        self.qos
    }

    pub fn set_qos(&mut self, qos: QosPolicies) {
        self.qos = qos
    }

    pub fn create_datawriter() {}
}
