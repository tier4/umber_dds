use crate::dds::participant::DomainParticipant;
use crate::dds::qos::QosPolicies;
use crate::dds::typedesc::TypeDesc;
use crate::structure::topic_kind::TopicKind;
use std::sync::Arc;

#[derive(Clone)]
pub struct Topic {
    inner: Arc<InnerTopic>,
}

impl Topic {
    pub fn new(
        name: String,
        type_desc: TypeDesc,
        my_domain_participant: DomainParticipant,
        my_qos_policies: QosPolicies,
        kind: TopicKind,
    ) -> Self {
        Self {
            inner: Arc::new(InnerTopic::new(
                name,
                type_desc,
                my_domain_participant,
                my_qos_policies,
                kind,
            )),
        }
    }

    pub fn name(&self) -> String {
        self.inner.name.clone()
    }
    pub fn type_desc(&self) -> TypeDesc {
        self.inner.type_desc.clone()
    }
    pub fn my_domain_participant(&self) -> DomainParticipant {
        self.inner.my_domain_participant.clone()
    }
    pub fn my_qos_policies(&self) -> QosPolicies {
        self.inner.my_qos_policies.clone()
    }
    pub fn kind(&self) -> TopicKind {
        self.inner.kind
    }
}

struct InnerTopic {
    name: String,
    type_desc: TypeDesc,
    my_domain_participant: DomainParticipant,
    my_qos_policies: QosPolicies,
    kind: TopicKind,
}

impl InnerTopic {
    fn new(
        name: String,
        type_desc: TypeDesc,
        my_domain_participant: DomainParticipant,
        my_qos_policies: QosPolicies,
        kind: TopicKind,
    ) -> Self {
        Self {
            name,
            type_desc,
            my_domain_participant,
            my_qos_policies,
            kind,
        }
    }
}
