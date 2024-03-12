#[derive(Clone, Copy)]
pub struct QosPolicies {
    durability: Option<policy::Durability>,
    presentation: Option<policy::Presentation>,
    deadline: Option<policy::Deadline>,
    latency_budget: Option<policy::LatencyBudget>,
    ownership: Option<policy::Ownership>,
    liveliness: Option<policy::Liveliness>,
    time_based_filter: Option<policy::TimeBasedFilter>,
    reliability: Option<policy::Reliability>,
    destination_order: Option<policy::DestinationOrder>,
    history: Option<policy::History>,
    resource_limits: Option<policy::ResourceLimits>,
    lifespan: Option<policy::Lifespan>,
}

// How to impl builder: https://keens.github.io/blog/2017/02/09/rustnochottoyarisuginabuilderpata_n/
#[derive(Default)]
pub struct QosBuilder {
    durability: Option<policy::Durability>,
    presentation: Option<policy::Presentation>,
    deadline: Option<policy::Deadline>,
    latency_budget: Option<policy::LatencyBudget>,
    ownership: Option<policy::Ownership>,
    liveliness: Option<policy::Liveliness>,
    time_based_filter: Option<policy::TimeBasedFilter>,
    reliability: Option<policy::Reliability>,
    destination_order: Option<policy::DestinationOrder>,
    history: Option<policy::History>,
    resource_limits: Option<policy::ResourceLimits>,
    lifespan: Option<policy::Lifespan>,
}

impl QosBuilder {
    pub fn new() -> Self {
        QosBuilder::default()
    }

    pub fn durability(mut self, durability: policy::Durability) -> Self {
        self.durability = Some(durability);
        self
    }

    pub fn presentation(mut self, presentation: policy::Presentation) -> Self {
        self.presentation = Some(presentation);
        self
    }

    pub fn deadline(mut self, deadline: policy::Deadline) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn latency_budget(mut self, latency_budget: policy::LatencyBudget) -> Self {
        self.latency_budget = Some(latency_budget);
        self
    }

    pub fn ownership(mut self, ownership: policy::Ownership) -> Self {
        self.ownership = Some(ownership);
        self
    }

    pub fn liveliness(mut self, liveliness: policy::Liveliness) -> Self {
        self.liveliness = Some(liveliness);
        self
    }

    pub fn time_based_filter(mut self, time_based_filter: policy::TimeBasedFilter) -> Self {
        self.time_based_filter = Some(time_based_filter);
        self
    }

    pub fn reliability(mut self, reliability: policy::Reliability) -> Self {
        self.reliability = Some(reliability);
        self
    }

    pub fn destination_order(mut self, destination_order: policy::DestinationOrder) -> Self {
        self.destination_order = Some(destination_order);
        self
    }

    pub fn history(mut self, history: policy::History) -> Self {
        self.history = Some(history);
        self
    }

    pub fn resource_limits(mut self, resource_limits: policy::ResourceLimits) -> Self {
        self.resource_limits = Some(resource_limits);
        self
    }

    pub fn lifespan(mut self, lifespan: policy::Lifespan) -> Self {
        self.lifespan = Some(lifespan);
        self
    }

    pub fn build(self) -> QosPolicies {
        QosPolicies {
            durability: self.durability,
            presentation: self.presentation,
            deadline: self.deadline,
            latency_budget: self.latency_budget,
            ownership: self.ownership,
            liveliness: self.liveliness,
            time_based_filter: self.time_based_filter,
            reliability: self.reliability,
            destination_order: self.destination_order,
            history: self.history,
            resource_limits: self.resource_limits,
            lifespan: self.lifespan,
        }
    }
}

pub mod policy {
    use crate::structure::duration::Duration;
    use serde::{
        ser::{SerializeStruct, Serializer},
        Deserialize, Serialize,
    };
    use serde_repr::{Deserialize_repr, Serialize_repr};
    use std::fmt;

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
        pub fn default_datareader() -> Self {
            Self {
                kind: ReliabilityQosKind::BestEffort,
                max_bloking_time: Duration {
                    seconds: 0,
                    fraction: 100,
                },
            }
        }
        pub fn default_datawriter() -> Self {
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
