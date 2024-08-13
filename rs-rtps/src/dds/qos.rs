// DDS 1.4 spec: 2.3.3 DCPS PSM : IDL
// How to impl builder: https://keens.github.io/blog/2017/02/09/rustnochottoyarisuginabuilderpata_n/

#[derive(Clone)]
pub enum DomainParticipantQos {
    Default,
    Policies(DomainParticipantQosPolicies),
}

#[derive(Clone)]
pub struct DomainParticipantQosPolicies {
    pub user_data: Option<policy::UserData>,
    pub entity_factory: Option<policy::EntityFactory>,
}

#[derive(Clone)]
pub enum TopicQos {
    Default,
    Policies(TopicQosPolicies),
}

#[derive(Clone)]
pub struct TopicQosPolicies {
    pub topic_data: Option<policy::TopicData>,
    pub durability: Option<policy::Durability>,
    pub durability_service: Option<policy::DurabilityService>,
    pub deadline: Option<policy::Deadline>,
    pub latency_budget: Option<policy::LatencyBudget>,
    pub liveliness: Option<policy::Liveliness>,
    pub reliability: Option<policy::Reliability>,
    pub destination_order: Option<policy::DestinationOrder>,
    pub history: Option<policy::History>,
    pub resource_limits: Option<policy::ResourceLimits>,
    pub transport_priority: Option<policy::TransportPriority>,
    pub lifespan: Option<policy::Lifespan>,
    pub ownership: Option<policy::Ownership>,
}

#[derive(Clone)]
pub enum DataWriterQos {
    Default,
    Policies(DataWriterQosPolicies),
}

#[derive(Clone)]
pub struct DataWriterQosPolicies {
    pub durability: Option<policy::Durability>,
    pub durability_service: Option<policy::DurabilityService>,
    pub deadline: Option<policy::Deadline>,
    pub latency_budget: Option<policy::LatencyBudget>,
    pub liveliness: Option<policy::Liveliness>,
    pub reliability: Option<policy::Reliability>,
    pub destination_order: Option<policy::DestinationOrder>,
    pub history: Option<policy::History>,
    pub resource_limits: Option<policy::ResourceLimits>,
    pub transport_priority: Option<policy::TransportPriority>,
    pub lifespan: Option<policy::Lifespan>,
    pub user_data: Option<policy::UserData>,
    pub ownership: Option<policy::Ownership>,
    pub ownership_strength: Option<policy::OwnershipStrength>,
    pub writer_data_lifecycle: Option<policy::WriterDataLifecycle>,
}

#[derive(Clone)]
pub enum PublisherQos {
    Default,
    Policies(PublisherQosPolicies),
}

#[derive(Clone)]
pub struct PublisherQosPolicies {
    pub presentation: Option<policy::Presentation>,
    pub partition: Option<policy::Partition>,
    pub group_data: Option<policy::GroupData>,
    pub entity_factory: Option<policy::EntityFactory>,
}

#[derive(Clone)]
pub enum DataReadedrQos {
    Default,
    Policies(DataReadedrQosPolicies),
}

#[derive(Clone)]
pub struct DataReadedrQosPolicies {
    pub durability: Option<policy::Durability>,
    pub deadline: Option<policy::Deadline>,
    pub latency_budget: Option<policy::LatencyBudget>,
    pub liveliness: Option<policy::Liveliness>,
    pub reliability: Option<policy::Reliability>,
    pub destination_order: Option<policy::DestinationOrder>,
    pub history: Option<policy::History>,
    pub resource_limits: Option<policy::ResourceLimits>,
    pub user_data: Option<policy::UserData>,
    pub ownership: Option<policy::Ownership>,
    pub time_based_filter: Option<policy::TimeBasedFilter>,
    pub reader_data_lifecycle: Option<policy::ReaderDataLifecycle>,
}

#[derive(Clone)]
pub enum SubscriberQos {
    Default,
    Policies(SubscriberQosPolicies),
}

#[derive(Clone)]
pub struct SubscriberQosPolicies {
    pub presentation: Option<policy::Presentation>,
    pub partition: Option<policy::Partition>,
    pub group_data: Option<policy::GroupData>,
    pub entity_factory: Option<policy::EntityFactory>,
}

macro_rules! builder_method {
    ($name:ident, $policy_name:ident) => {
        pub fn $name(mut self, $name: policy::$policy_name) -> Self {
            self.$name = Some($name);
            self
        }
    };
}

#[derive(Default)]
pub struct DomainParticipantQosBuilder {
    pub user_data: Option<policy::UserData>,
    pub entity_factory: Option<policy::EntityFactory>,
}

impl DomainParticipantQosBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    builder_method!(user_data, UserData);
    builder_method!(entity_factory, EntityFactory);

    pub fn build(self) -> DomainParticipantQosPolicies {
        DomainParticipantQosPolicies {
            user_data: self.user_data,
            entity_factory: self.entity_factory,
        }
    }
}

#[derive(Default)]
pub struct TopicQosBuilder {
    pub topic_data: Option<policy::TopicData>,
    pub durability: Option<policy::Durability>,
    pub durability_service: Option<policy::DurabilityService>,
    pub deadline: Option<policy::Deadline>,
    pub latency_budget: Option<policy::LatencyBudget>,
    pub liveliness: Option<policy::Liveliness>,
    pub reliability: Option<policy::Reliability>,
    pub destination_order: Option<policy::DestinationOrder>,
    pub history: Option<policy::History>,
    pub resource_limits: Option<policy::ResourceLimits>,
    pub transport_priority: Option<policy::TransportPriority>,
    pub lifespan: Option<policy::Lifespan>,
    pub ownership: Option<policy::Ownership>,
}

impl TopicQosBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    builder_method!(topic_data, TopicData);
    builder_method!(durability, Durability);
    builder_method!(durability_service, DurabilityService);
    builder_method!(deadline, Deadline);
    builder_method!(latency_budget, LatencyBudget);
    builder_method!(liveliness, Liveliness);
    builder_method!(reliability, Reliability);
    builder_method!(destination_order, DestinationOrder);
    builder_method!(history, History);
    builder_method!(resource_limits, ResourceLimits);
    builder_method!(transport_priority, TransportPriority);
    builder_method!(lifespan, Lifespan);
    builder_method!(ownership, Ownership);

    pub fn build(self) -> TopicQosPolicies {
        TopicQosPolicies {
            topic_data: self.topic_data,
            durability: self.durability,
            durability_service: self.durability_service,
            deadline: self.deadline,
            latency_budget: self.latency_budget,
            liveliness: self.liveliness,
            reliability: self.reliability,
            destination_order: self.destination_order,
            history: self.history,
            resource_limits: self.resource_limits,
            transport_priority: self.transport_priority,
            lifespan: self.lifespan,
            ownership: self.ownership,
        }
    }
}

#[derive(Default)]
pub struct DataWriterQosBuilder {
    pub durability: Option<policy::Durability>,
    pub durability_service: Option<policy::DurabilityService>,
    pub deadline: Option<policy::Deadline>,
    pub latency_budget: Option<policy::LatencyBudget>,
    pub liveliness: Option<policy::Liveliness>,
    pub reliability: Option<policy::Reliability>,
    pub destination_order: Option<policy::DestinationOrder>,
    pub history: Option<policy::History>,
    pub resource_limits: Option<policy::ResourceLimits>,
    pub transport_priority: Option<policy::TransportPriority>,
    pub lifespan: Option<policy::Lifespan>,
    pub user_data: Option<policy::UserData>,
    pub ownership: Option<policy::Ownership>,
    pub ownership_strength: Option<policy::OwnershipStrength>,
    pub writer_data_lifecycle: Option<policy::WriterDataLifecycle>,
}

impl DataWriterQosBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    builder_method!(durability, Durability);
    builder_method!(durability_service, DurabilityService);
    builder_method!(deadline, Deadline);
    builder_method!(latency_budget, LatencyBudget);
    builder_method!(liveliness, Liveliness);
    builder_method!(reliability, Reliability);
    builder_method!(destination_order, DestinationOrder);
    builder_method!(history, History);
    builder_method!(resource_limits, ResourceLimits);
    builder_method!(transport_priority, TransportPriority);
    builder_method!(lifespan, Lifespan);
    builder_method!(user_data, UserData);
    builder_method!(ownership, Ownership);
    builder_method!(ownership_strength, OwnershipStrength);
    builder_method!(writer_data_lifecycle, WriterDataLifecycle);

    pub fn build(self) -> DataWriterQosPolicies {
        DataWriterQosPolicies {
            durability: self.durability,
            durability_service: self.durability_service,
            deadline: self.deadline,
            latency_budget: self.latency_budget,
            liveliness: self.liveliness,
            reliability: self.reliability,
            destination_order: self.destination_order,
            history: self.history,
            resource_limits: self.resource_limits,
            transport_priority: self.transport_priority,
            lifespan: self.lifespan,
            user_data: self.user_data,
            ownership: self.ownership,
            ownership_strength: self.ownership_strength,
            writer_data_lifecycle: self.writer_data_lifecycle,
        }
    }
}

#[derive(Default)]
pub struct PublisherQosBuilder {
    pub presentation: Option<policy::Presentation>,
    pub partition: Option<policy::Partition>,
    pub group_data: Option<policy::GroupData>,
    pub entity_factory: Option<policy::EntityFactory>,
}

impl PublisherQosBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    builder_method!(presentation, Presentation);
    builder_method!(partition, Partition);
    builder_method!(group_data, GroupData);
    builder_method!(entity_factory, EntityFactory);

    pub fn build(self) -> PublisherQosPolicies {
        PublisherQosPolicies {
            presentation: self.presentation,
            partition: self.partition,
            group_data: self.group_data,
            entity_factory: self.entity_factory,
        }
    }
}

#[derive(Default)]
pub struct DataReadedrQosBuilder {
    pub durability: Option<policy::Durability>,
    pub deadline: Option<policy::Deadline>,
    pub latency_budget: Option<policy::LatencyBudget>,
    pub liveliness: Option<policy::Liveliness>,
    pub reliability: Option<policy::Reliability>,
    pub destination_order: Option<policy::DestinationOrder>,
    pub history: Option<policy::History>,
    pub resource_limits: Option<policy::ResourceLimits>,
    pub user_data: Option<policy::UserData>,
    pub ownership: Option<policy::Ownership>,
    pub time_based_filter: Option<policy::TimeBasedFilter>,
    pub reader_data_lifecycle: Option<policy::ReaderDataLifecycle>,
}

impl DataReadedrQosBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    builder_method!(durability, Durability);
    builder_method!(deadline, Deadline);
    builder_method!(latency_budget, LatencyBudget);
    builder_method!(reliability, Reliability);
    builder_method!(destination_order, DestinationOrder);
    builder_method!(history, History);
    builder_method!(resource_limits, ResourceLimits);
    builder_method!(user_data, UserData);
    builder_method!(ownership, Ownership);
    builder_method!(time_based_filter, TimeBasedFilter);
    builder_method!(reader_data_lifecycle, ReaderDataLifecycle);

    pub fn build(self) -> DataReadedrQosPolicies {
        DataReadedrQosPolicies {
            durability: self.durability,
            deadline: self.deadline,
            latency_budget: self.latency_budget,
            liveliness: self.liveliness,
            reliability: self.reliability,
            destination_order: self.destination_order,
            history: self.history,
            resource_limits: self.resource_limits,
            user_data: self.user_data,
            ownership: self.ownership,
            time_based_filter: self.time_based_filter,
            reader_data_lifecycle: self.reader_data_lifecycle,
        }
    }
}

#[derive(Default)]
pub struct SubscriberQosBuilder {
    pub presentation: Option<policy::Presentation>,
    pub partition: Option<policy::Partition>,
    pub group_data: Option<policy::GroupData>,
    pub entity_factory: Option<policy::EntityFactory>,
}

impl SubscriberQosBuilder {
    pub fn new() -> Self {
        SubscriberQosBuilder::default()
    }

    builder_method!(presentation, Presentation);
    builder_method!(partition, Partition);
    builder_method!(group_data, GroupData);
    builder_method!(entity_factory, EntityFactory);

    pub fn build(self) -> SubscriberQosPolicies {
        SubscriberQosPolicies {
            presentation: self.presentation,
            partition: self.partition,
            group_data: self.group_data,
            entity_factory: self.entity_factory,
        }
    }
}

pub mod policy {
    use crate::structure::duration::Duration;
    use serde::{Deserialize, Serialize};
    use serde_repr::{Deserialize_repr, Serialize_repr};

    // Default value of QoS Policies is on DDS v1.4 spec 2.2.3 Supported QoS
    const LENGTH_UNLIMITED: i32 = -1;

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct DurabilityService {
        lease_duration: Duration,
        history_kind: HistoryQosKind,
        history_depth: i32,
        max_samples: i32,
        max_instance: i32,
        max_samples_per_instanse: i32,
    }
    impl Default for DurabilityService {
        fn default() -> Self {
            Self {
                lease_duration: Duration {
                    seconds: 100,
                    fraction: 0,
                },
                history_kind: HistoryQosKind::KeepLast,
                history_depth: 1,
                max_samples: LENGTH_UNLIMITED,
                max_instance: LENGTH_UNLIMITED,
                max_samples_per_instanse: LENGTH_UNLIMITED,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr)]
    #[repr(i32)]
    pub enum Durability {
        Volatile = 0,
        TransientLocal = 1,
        Transient = 2,
        Persistent = 3,
    }
    impl Default for Durability {
        fn default() -> Self {
            Self::Volatile
        }
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct Presentation {
        pub access_scope: PresentationQosAccessScopeKind,
        pub coherent_access: bool,
        pub ordered_access: bool,
    }
    impl Default for Presentation {
        fn default() -> Self {
            Self {
                access_scope: PresentationQosAccessScopeKind::default(),
                coherent_access: false,
                ordered_access: false,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr)]
    #[repr(i32)]
    pub enum PresentationQosAccessScopeKind {
        Instance = 0,
        Topic = 1,
        Group = 2,
    }
    impl Default for PresentationQosAccessScopeKind {
        fn default() -> Self {
            Self::Instance
        }
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct Deadline {
        pub period: Duration,
    }
    impl Default for Deadline {
        fn default() -> Self {
            Self {
                period: Duration::INFINITE,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct LatencyBudget(pub Duration);
    impl Default for LatencyBudget {
        fn default() -> Self {
            Self(Duration::ZERO)
        }
    }

    #[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr)]
    #[repr(i32)]
    pub enum Ownership {
        Shared = 0,
        Exclusive = 1,
    }
    impl Default for Ownership {
        fn default() -> Self {
            Self::Shared
        }
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct OwnershipStrength(pub i32);
    impl Default for OwnershipStrength {
        fn default() -> Self {
            Self(0)
        }
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct Liveliness {
        pub kind: LivelinessQosKind,
        pub lease_duration: Duration,
    }
    impl Default for Liveliness {
        fn default() -> Self {
            Self {
                kind: LivelinessQosKind::Automatic,
                lease_duration: Duration::INFINITE,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr)]
    #[repr(i32)]
    pub enum LivelinessQosKind {
        Automatic = 0,
        ManualByParticipant = 1,
        ManualByTopic = 2,
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct TimeBasedFilter {
        pub minimun_separation: Duration,
    }
    impl Default for TimeBasedFilter {
        fn default() -> Self {
            Self {
                minimun_separation: Duration::ZERO,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct Reliability {
        pub kind: ReliabilityQosKind,
        pub max_bloking_time: Duration,
    }
    impl Reliability {
        // DDS v1.4 spec, 2.2.3 Supported QoS specifies
        // default value of max_bloking_time is 100ms

        pub fn default_besteffort() -> Self {
            Self {
                kind: ReliabilityQosKind::BestEffort,
                max_bloking_time: Duration {
                    seconds: 0,
                    fraction: 100,
                },
            }
        }
        pub fn default_reliable() -> Self {
            Self {
                kind: ReliabilityQosKind::Reliable,
                max_bloking_time: Duration {
                    seconds: 0,
                    fraction: 100,
                },
            }
        }
    }

    #[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr)]
    #[repr(i32)]
    pub enum ReliabilityQosKind {
        Reliable = 2,
        BestEffort = 1,
    }

    #[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr)]
    #[repr(i32)]
    pub enum DestinationOrder {
        ByReceptionTimestamp = 0,
        BySourceTimestamp = 1,
    }
    impl Default for DestinationOrder {
        fn default() -> Self {
            Self::ByReceptionTimestamp
        }
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct History {
        pub kind: HistoryQosKind,
        pub depth: i32,
    }
    impl Default for History {
        fn default() -> Self {
            Self {
                kind: HistoryQosKind::KeepLast,
                depth: 1,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Serialize_repr, Deserialize_repr)]
    #[repr(i32)]
    pub enum HistoryQosKind {
        KeepLast = 0,
        KeepAll = 1,
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct ResourceLimits {
        pub max_samples: i32,
        pub max_instance: i32,
        pub max_samples_per_instanse: i32,
    }
    impl Default for ResourceLimits {
        fn default() -> Self {
            Self {
                max_samples: LENGTH_UNLIMITED,
                max_instance: LENGTH_UNLIMITED,
                max_samples_per_instanse: LENGTH_UNLIMITED,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct Lifespan(pub Duration);
    impl Default for Lifespan {
        fn default() -> Self {
            Self(Duration::INFINITE)
        }
    }

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct Partition {
        pub name: Vec<String>,
    }
    impl Partition {
        pub fn serialized_size(&self) -> u16 {
            // length of name: i32, 4 octet
            let mut len = 4;
            for n in &self.name {
                len += 4; // length of n: i32, 4octet
                len += (n.len() as u16) + 1; // length of n+1: `+1` means null char
                match len % 4 {
                    // padding
                    0 => (),
                    1 => len += 3,
                    2 => len += 2,
                    3 => len += 1,
                    _ => unreachable!(),
                }
            }
            len
        }
    }

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct UserData {
        pub value: Vec<u8>,
    }
    impl UserData {
        pub fn serialized_size(&self) -> u16 {
            4 + self.value.len() as u16
        }
    }

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct TopicData {
        pub value: Vec<u8>,
    }
    impl TopicData {
        pub fn serialized_size(&self) -> u16 {
            4 + self.value.len() as u16
        }
    }

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct GroupData {
        pub value: Vec<u8>,
    }
    impl GroupData {
        pub fn serialized_size(&self) -> u16 {
            4 + self.value.len() as u16
        }
    }

    #[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
    pub struct WriterDataLifecycle {
        pub autodispose_unregistered_instance: bool,
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct ReaderDataLifecycle {
        pub autopurge_nowriter_samples_delay: Duration,
        pub autopurge_dispose_samples_delay: Duration,
    }
    impl Default for ReaderDataLifecycle {
        fn default() -> Self {
            Self {
                autopurge_dispose_samples_delay: Duration::INFINITE,
                autopurge_nowriter_samples_delay: Duration::INFINITE,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
    pub struct TransportPriority {
        pub value: i32,
    }

    #[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
    pub struct EntityFactory {
        pub autoenable_created_entites: bool,
    }
}

mod test {
    use super::policy;
    use crate::structure::duration::Duration;
    use cdr::{Infinite, PlCdrLe};

    #[test]
    fn test_serialize() {
        let history = policy::History {
            kind: policy::HistoryQosKind::KeepAll,
            depth: 100,
        };
        let serialized = cdr::serialize::<_, _, PlCdrLe>(&history, Infinite).unwrap();
        let mut serialized_str = String::new();
        let mut count = 0;
        for b in serialized {
            serialized_str += &format!("{:>02X} ", b);
            count += 1;
            if count % 16 == 0 {
                serialized_str += "\n";
            } else if count % 8 == 0 {
                serialized_str += " ";
            }
        }
        eprintln!("~~~~~~~~~~~~~~~~~~\n{}\n~~~~~~~~~~~~~~~~~~", serialized_str);
    }
    #[test]
    fn test_deserialize() {
        let presentation = policy::Presentation {
            access_scope: policy::PresentationQosAccessScopeKind::Topic,
            coherent_access: false,
            ordered_access: true,
        };
        let serialized = cdr::serialize::<_, _, PlCdrLe>(&presentation, Infinite).unwrap();
        let deserialized = cdr::deserialize::<policy::Presentation>(&serialized).unwrap();
        match deserialized.access_scope {
            policy::PresentationQosAccessScopeKind::Topic => (),
            _ => panic!(),
        }
        assert_eq!(presentation.coherent_access, deserialized.coherent_access);
        assert_eq!(presentation.ordered_access, deserialized.ordered_access);
    }
}
