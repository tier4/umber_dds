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
