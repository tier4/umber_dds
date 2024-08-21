use crate::dds::participant::DomainParticipant;
use crate::dds::qos::TopicQosPolicies;
use crate::discovery::structure::data::{
    PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
};
use crate::structure::topic_kind::TopicKind;
use std::sync::Arc;

#[derive(Clone)]
pub struct Topic {
    inner: Arc<InnerTopic>,
}

impl Topic {
    pub fn new(
        name: String,
        type_desc: String,
        my_domain_participant: DomainParticipant,
        my_qos_policies: TopicQosPolicies,
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
    pub fn type_desc(&self) -> String {
        self.inner.type_desc.clone()
    }
    pub fn my_domain_participant(&self) -> DomainParticipant {
        self.inner.my_domain_participant.clone()
    }
    pub fn my_qos_policies(&self) -> TopicQosPolicies {
        self.inner.my_qos_policies.clone()
    }
    pub fn kind(&self) -> TopicKind {
        self.inner.kind
    }

    pub fn sub_builtin_topic_data(&self) -> SubscriptionBuiltinTopicData {
        self.inner.sub_builtin_topic_data()
    }
    pub fn pub_builtin_topic_data(&self) -> PublicationBuiltinTopicData {
        self.inner.pub_builtin_topic_data()
    }
}

struct InnerTopic {
    name: String,
    type_desc: String,
    my_domain_participant: DomainParticipant,
    my_qos_policies: TopicQosPolicies,
    kind: TopicKind,
}

impl InnerTopic {
    fn new(
        name: String,
        type_desc: String,
        my_domain_participant: DomainParticipant,
        my_qos_policies: TopicQosPolicies,
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

    fn pub_builtin_topic_data(&self) -> PublicationBuiltinTopicData {
        PublicationBuiltinTopicData::new(
            None,
            None,
            Some(self.name.clone()),
            Some(self.type_desc.clone()),
            Some(self.my_qos_policies.durability),
            None,
            Some(self.my_qos_policies.deadline),
            Some(self.my_qos_policies.latency_budget),
            Some(self.my_qos_policies.liveliness),
            Some(self.my_qos_policies.reliability),
            Some(self.my_qos_policies.lifespan),
            None,
            None,
            Some(self.my_qos_policies.ownership),
            None,
            Some(self.my_qos_policies.destination_order),
            None,
            None,
            None,
            None,
        )
    }
    fn sub_builtin_topic_data(&self) -> SubscriptionBuiltinTopicData {
        /*
         * if SubscriptionBuiltinTopicData include liveliness and deadline QoS Policy,
         * FastDDS say detects incompatible QoS.
         * TODO: fix it
         */
        SubscriptionBuiltinTopicData::new(
            None,
            None,
            Some(self.name.clone()),
            Some(self.type_desc.clone()),
            Some(self.my_qos_policies.durability),
            None, // Some(self.my_qos_policies.deadline),
            Some(self.my_qos_policies.latency_budget),
            None, // Some(self.my_qos_policies.liveliness),
            Some(self.my_qos_policies.reliability),
            Some(self.my_qos_policies.ownership),
            Some(self.my_qos_policies.destination_order),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(self.my_qos_policies.lifespan),
        )
    }
}
