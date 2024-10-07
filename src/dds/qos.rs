//! set of DDS QoS policies for each Entity and its builder

// DDS 1.4 spec: 2.3.3 DCPS PSM : IDL
// How to impl builder: https://keens.github.io/blog/2017/02/09/rustnochottoyarisuginabuilderpata_n/

use policy::*;

/// for setting QoS on a DomainParticipant
#[derive(Clone)]
pub enum DomainParticipantQos {
    /// represent default QoS of DomainParticipant.
    Default,
    Policies(DomainParticipantQosPolicies),
}

/// A collection of QoS policies for configuring the behavior of a DataReader
#[derive(Clone)]
pub struct DomainParticipantQosPolicies {
    pub user_data: UserData,
    pub entity_factory: EntityFactory,
}

/// for setting QoS on a Topic
#[derive(Clone)]
pub enum TopicQos {
    /// represent default QoS of Topic.
    ///
    /// it can get `DomainParticipant::get_defaul_topict_qos()` and
    /// change `DomainParticipant::set_default_topic_qos()`
    Default,
    Policies(TopicQosPolicies),
}

/// A collection of QoS policies for configuring the behavior of a Topic
#[derive(Clone)]
pub struct TopicQosPolicies {
    pub topic_data: TopicData,
    pub durability: Durability,
    pub durability_service: DurabilityService,
    pub deadline: Deadline,
    pub latency_budget: LatencyBudget,
    pub liveliness: Liveliness,
    pub reliability: Reliability,
    pub destination_order: DestinationOrder,
    pub history: History,
    pub resource_limits: ResourceLimits,
    pub transport_priority: TransportPriority,
    pub lifespan: Lifespan,
    pub ownership: Ownership,
}

/// for setting QoS on a DataWriter
#[derive(Clone)]
pub enum DataWriterQos {
    /// represent default QoS of DataWriter.
    ///
    /// it can get `Publisher::get_default_datawriter_qos()` and
    /// change `Publisher::set_default_datawriter_qos()`
    Default,
    Policies(DataWriterQosPolicies),
}

/// A collection of QoS policies for configuring the behavior of a DataWriter
#[derive(Clone)]
pub struct DataWriterQosPolicies {
    pub durability: Durability,
    pub durability_service: DurabilityService,
    pub deadline: Deadline,
    pub latency_budget: LatencyBudget,
    pub liveliness: Liveliness,
    pub reliability: Reliability,
    pub destination_order: DestinationOrder,
    pub history: History,
    pub resource_limits: ResourceLimits,
    pub transport_priority: TransportPriority,
    pub lifespan: Lifespan,
    pub user_data: UserData,
    pub ownership: Ownership,
    pub ownership_strength: OwnershipStrength,
    pub writer_data_lifecycle: WriterDataLifecycle,
}

impl DataWriterQosPolicies {
    pub fn is_compatible(&self, qos: &DataReadedrQosPolicies) -> Result<(), String> {
        let mut msg = String::new();
        if !Durability::is_compatible(self.durability, qos.durability) {
            msg += &format!(
                "durability is not compatible. self: {:?}, writer: {:?}\n",
                self.durability, qos.durability
            );
        } else if !Deadline::is_compatible(self.deadline, qos.deadline) {
            msg += &format!(
                "deadline is not compatible. self: {:?}, writer: {:?}\n",
                self.deadline, qos.deadline
            );
        } else if !LatencyBudget::is_compatible(self.latency_budget, qos.latency_budget) {
            msg += &format!(
                "latency_budget is not compatible. self: {:?}, writer: {:?}\n",
                self.latency_budget, qos.latency_budget
            );
        } else if !Ownership::is_compatible(self.ownership, qos.ownership) {
            msg += &format!(
                "ownership is not compatible. self: {:?}, writer: {:?}\n",
                self.ownership, qos.ownership
            );
        } else if !Liveliness::is_compatible(self.liveliness, qos.liveliness) {
            msg += &format!(
                "liveliness is not compatible. self: {:?}, writer: {:?}\n",
                self.liveliness, qos.liveliness
            );
        } else if !Reliability::is_compatible(self.reliability, qos.reliability) {
            msg += &format!(
                "reliability is not compatible. self: {:?}, writer: {:?}\n",
                self.reliability, qos.reliability
            );
        } else if !DestinationOrder::is_compatible(self.destination_order, qos.destination_order) {
            msg += &format!(
                "destination_order is not compatible. self: {:?}, writer: {:?}\n",
                self.destination_order, qos.destination_order
            );
        } else {
            return Ok(());
        }
        Err(msg)
    }
}

/// for setting QoS on a Publisher
#[derive(Clone)]
pub enum PublisherQos {
    /// represent default QoS of Publisher.
    ///
    /// it can get `DomainParticipant::get_default_publisher_qos()` and
    /// change `DomainParticipant::set_default_publisher_qos()`
    Default,
    Policies(PublisherQosPolicies),
}

/// A collection of QoS policies for configuring the behavior of a Publisher
#[derive(Clone)]
pub struct PublisherQosPolicies {
    pub presentation: Presentation,
    pub partition: Partition,
    pub group_data: GroupData,
    pub entity_factory: EntityFactory,
}

/// for setting QoS on a DataReadedr
#[derive(Clone)]
pub enum DataReadedrQos {
    /// represent default QoS of DataReadedr.
    ///
    /// it can get `Subscriber::get_default_datareader_qos()` and
    /// change `Subscriber::set_default_datareader_qos()`
    Default,
    Policies(DataReadedrQosPolicies),
}

/// A collection of QoS policies for configuring the behavior of a DataReader
#[derive(Clone)]
pub struct DataReadedrQosPolicies {
    pub durability: Durability,
    pub deadline: Deadline,
    pub latency_budget: LatencyBudget,
    pub liveliness: Liveliness,
    pub reliability: Reliability,
    pub destination_order: DestinationOrder,
    pub history: History,
    pub resource_limits: ResourceLimits,
    pub user_data: UserData,
    pub ownership: Ownership,
    pub time_based_filter: TimeBasedFilter,
    pub reader_data_lifecycle: ReaderDataLifecycle,
}
impl DataReadedrQosPolicies {
    pub fn is_compatible(&self, qos: &DataWriterQosPolicies) -> Result<(), String> {
        let mut msg = String::new();
        if !Durability::is_compatible(qos.durability, self.durability) {
            msg += &format!(
                "durability is not compatible. self: {:?}, writer: {:?}\n",
                self.durability, qos.durability
            );
        } else if !Deadline::is_compatible(qos.deadline, self.deadline) {
            msg += &format!(
                "deadline is not compatible. self: {:?}, writer: {:?}\n",
                self.deadline, qos.deadline
            );
        } else if !LatencyBudget::is_compatible(qos.latency_budget, self.latency_budget) {
            msg += &format!(
                "latency_budget is not compatible. self: {:?}, writer: {:?}\n",
                self.latency_budget, qos.latency_budget
            );
        } else if !Ownership::is_compatible(qos.ownership, self.ownership) {
            msg += &format!(
                "ownership is not compatible. self: {:?}, writer: {:?}\n",
                self.ownership, qos.ownership
            );
        } else if !Liveliness::is_compatible(qos.liveliness, self.liveliness) {
            msg += &format!(
                "liveliness is not compatible. self: {:?}, writer: {:?}\n",
                self.liveliness, qos.liveliness
            );
        } else if !Reliability::is_compatible(qos.reliability, self.reliability) {
            msg += &format!(
                "reliability is not compatible. self: {:?}, writer: {:?}\n",
                self.reliability, qos.reliability
            );
        } else if !DestinationOrder::is_compatible(qos.destination_order, self.destination_order) {
            msg += &format!(
                "destination_order is not compatible. self: {:?}, writer: {:?}\n",
                self.destination_order, qos.destination_order
            );
        } else {
            return Ok(());
        }
        Err(msg)
    }
}

/// for setting QoS on a Subscriber
#[derive(Clone)]
pub enum SubscriberQos {
    /// represent default QoS of Subscriber.
    ///
    /// it can get `DomainParticipant::get_default_subscriber_qos()` and
    /// change `DomainParticipant::set_default_subscriber_qos()`
    Default,
    Policies(SubscriberQosPolicies),
}

/// A collection of QoS policies for configuring the behavior of a Subscriber
#[derive(Clone)]
pub struct SubscriberQosPolicies {
    pub presentation: Presentation,
    pub partition: Partition,
    pub group_data: GroupData,
    pub entity_factory: EntityFactory,
}

macro_rules! builder_method {
    ($name:ident, $policy_name:ident) => {
        pub fn $name(mut self, $name: $policy_name) -> Self {
            self.$name = Some($name);
            self
        }
    };
}

/// Builder of DomainParticipantQosPolicies
#[derive(Default)]
pub struct DomainParticipantQosBuilder {
    user_data: Option<UserData>,
    entity_factory: Option<EntityFactory>,
}

impl DomainParticipantQosBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    builder_method!(user_data, UserData);
    builder_method!(entity_factory, EntityFactory);

    pub fn build(self) -> DomainParticipantQosPolicies {
        DomainParticipantQosPolicies {
            user_data: self.user_data.unwrap_or_default(),
            entity_factory: self.entity_factory.unwrap_or_default(),
        }
    }
}

/// Builder of TopicQosPolicies
#[derive(Default)]
pub struct TopicQosBuilder {
    topic_data: Option<TopicData>,
    durability: Option<Durability>,
    durability_service: Option<DurabilityService>,
    deadline: Option<Deadline>,
    latency_budget: Option<LatencyBudget>,
    liveliness: Option<Liveliness>,
    reliability: Option<Reliability>,
    destination_order: Option<DestinationOrder>,
    history: Option<History>,
    resource_limits: Option<ResourceLimits>,
    transport_priority: Option<TransportPriority>,
    lifespan: Option<Lifespan>,
    ownership: Option<Ownership>,
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
            topic_data: self.topic_data.unwrap_or_default(),
            durability: self.durability.unwrap_or_default(),
            durability_service: self.durability_service.unwrap_or_default(),
            deadline: self.deadline.unwrap_or_default(),
            latency_budget: self.latency_budget.unwrap_or_default(),
            liveliness: self.liveliness.unwrap_or_default(),
            reliability: self
                .reliability
                .unwrap_or(Reliability::default_besteffort()),
            destination_order: self.destination_order.unwrap_or_default(),
            history: self.history.unwrap_or_default(),
            resource_limits: self.resource_limits.unwrap_or_default(),
            transport_priority: self.transport_priority.unwrap_or_default(),
            lifespan: self.lifespan.unwrap_or_default(),
            ownership: self.ownership.unwrap_or_default(),
        }
    }
}

/// Builder of DataWriterQosPolicies
#[derive(Default)]
pub struct DataWriterQosBuilder {
    durability: Option<Durability>,
    durability_service: Option<DurabilityService>,
    deadline: Option<Deadline>,
    latency_budget: Option<LatencyBudget>,
    liveliness: Option<Liveliness>,
    reliability: Option<Reliability>,
    destination_order: Option<DestinationOrder>,
    history: Option<History>,
    resource_limits: Option<ResourceLimits>,
    transport_priority: Option<TransportPriority>,
    lifespan: Option<Lifespan>,
    user_data: Option<UserData>,
    ownership: Option<Ownership>,
    ownership_strength: Option<OwnershipStrength>,
    writer_data_lifecycle: Option<WriterDataLifecycle>,
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
            durability: self.durability.unwrap_or_default(),
            durability_service: self.durability_service.unwrap_or_default(),
            deadline: self.deadline.unwrap_or_default(),
            latency_budget: self.latency_budget.unwrap_or_default(),
            liveliness: self.liveliness.unwrap_or_default(),
            reliability: self.reliability.unwrap_or(Reliability::default_reliable()),
            destination_order: self.destination_order.unwrap_or_default(),
            history: self.history.unwrap_or_default(),
            resource_limits: self.resource_limits.unwrap_or_default(),
            transport_priority: self.transport_priority.unwrap_or_default(),
            lifespan: self.lifespan.unwrap_or_default(),
            user_data: self.user_data.unwrap_or_default(),
            ownership: self.ownership.unwrap_or_default(),
            ownership_strength: self.ownership_strength.unwrap_or_default(),
            writer_data_lifecycle: self.writer_data_lifecycle.unwrap_or_default(),
        }
    }
}

/// Builder of PublisherQosPolicies
#[derive(Default)]
pub struct PublisherQosBuilder {
    presentation: Option<Presentation>,
    partition: Option<Partition>,
    group_data: Option<GroupData>,
    entity_factory: Option<EntityFactory>,
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
            presentation: self.presentation.unwrap_or_default(),
            partition: self.partition.unwrap_or_default(),
            group_data: self.group_data.unwrap_or_default(),
            entity_factory: self.entity_factory.unwrap_or_default(),
        }
    }
}

/// Builder of DataReadedrQosPolicies
#[derive(Default)]
pub struct DataReadedrQosBuilder {
    durability: Option<Durability>,
    deadline: Option<Deadline>,
    latency_budget: Option<LatencyBudget>,
    liveliness: Option<Liveliness>,
    reliability: Option<Reliability>,
    destination_order: Option<DestinationOrder>,
    history: Option<History>,
    resource_limits: Option<ResourceLimits>,
    user_data: Option<UserData>,
    ownership: Option<Ownership>,
    time_based_filter: Option<TimeBasedFilter>,
    reader_data_lifecycle: Option<ReaderDataLifecycle>,
}

impl DataReadedrQosBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    builder_method!(durability, Durability);
    builder_method!(deadline, Deadline);
    builder_method!(latency_budget, LatencyBudget);
    builder_method!(liveliness, Liveliness);
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
            durability: self.durability.unwrap_or_default(),
            deadline: self.deadline.unwrap_or_default(),
            latency_budget: self.latency_budget.unwrap_or_default(),
            liveliness: self.liveliness.unwrap_or_default(),
            reliability: self
                .reliability
                .unwrap_or(Reliability::default_besteffort()),
            destination_order: self.destination_order.unwrap_or_default(),
            history: self.history.unwrap_or_default(),
            resource_limits: self.resource_limits.unwrap_or_default(),
            user_data: self.user_data.unwrap_or_default(),
            ownership: self.ownership.unwrap_or_default(),
            time_based_filter: self.time_based_filter.unwrap_or_default(),
            reader_data_lifecycle: self.reader_data_lifecycle.unwrap_or_default(),
        }
    }
}

/// Builder of SubscriberQosPolicies
#[derive(Default)]
pub struct SubscriberQosBuilder {
    presentation: Option<Presentation>,
    partition: Option<Partition>,
    group_data: Option<GroupData>,
    entity_factory: Option<EntityFactory>,
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
            presentation: self.presentation.unwrap_or_default(),
            partition: self.partition.unwrap_or_default(),
            group_data: self.group_data.unwrap_or_default(),
            entity_factory: self.entity_factory.unwrap_or_default(),
        }
    }
}

pub mod policy {
    //! DDS QoS policies
    //!
    //! For more details on each QoS policy, please refer to the DDS specification.
    //! DDS v1.4 spec, 2.2.3 Supported QoS (<https://www.omg.org/spec/DDS/1.4/PDF#G5.1034386>)
    use crate::structure::Duration;
    use serde::{Deserialize, Serialize};
    use serde_repr::{Deserialize_repr, Serialize_repr};

    // Default value of QoS Policies is on DDS v1.4 spec 2.2.3 Supported QoS
    const LENGTH_UNLIMITED: i32 = -1;

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct DurabilityService {
        service_cleanup_delay: Duration,
        history_kind: HistoryQosKind,
        history_depth: i32,
        max_samples: i32,
        max_instance: i32,
        max_samples_per_instanse: i32,
    }
    impl Default for DurabilityService {
        fn default() -> Self {
            Self {
                service_cleanup_delay: Duration::ZERO,
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
    /// rtps 2.3 spec, 8.7.2.2 DDS QoS Parameters that affect the wire protoco
    /// 8.7.2.2.1 DURABILITY
    /// > While volatile and transient-local durability do not affect the RTPS protocol,
    /// > support for transient and persistent durability may.
    /// UmberDDS don't support optional Durability QoS value "Transient" and "Persistent".
    /// So, this config dosen't affect behavior.
    pub enum Durability {
        Volatile = 0,
        TransientLocal = 1,
        // Transient = 2, // DDS spec say Support this is optional
        // Persistent = 3, // DDS spec say Support this is optional
    }
    impl Durability {
        /// offered is Publisher side QoS value
        /// requested is Subscriber side QoS value
        pub(crate) fn is_compatible(offered: Self, requested: Self) -> bool {
            offered as usize >= requested as usize
        }
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
    impl Presentation {
        /// offered is Publisher side QoS value
        /// requested is Subscriber side QoS value
        pub(crate) fn is_compatible(offered: Self, requested: Self) -> bool {
            offered.access_scope as usize >= requested.access_scope as usize
        }
    }
    #[allow(clippy::derivable_impls)]
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
    impl Deadline {
        /// offered is Publisher side QoS value
        /// requested is Subscriber side QoS value
        pub(crate) fn is_compatible(offered: Self, requested: Self) -> bool {
            requested.period == Self::default().period || offered.period <= requested.period
        }
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
    impl LatencyBudget {
        /// offered is Publisher side QoS value
        /// requested is Subscriber side QoS value
        pub(crate) fn is_compatible(offered: Self, requested: Self) -> bool {
            offered.0 <= requested.0
        }
    }
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
    impl Ownership {
        /// offered is Publisher side QoS value
        /// requested is Subscriber side QoS value
        pub(crate) fn is_compatible(offered: Self, requested: Self) -> bool {
            offered as usize == requested as usize
        }
    }
    impl Default for Ownership {
        fn default() -> Self {
            Self::Shared
        }
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct OwnershipStrength(pub i32);
    #[allow(clippy::derivable_impls)]
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
    impl Liveliness {
        /// offered is Publisher side QoS value
        /// requested is Subscriber side QoS value
        pub(crate) fn is_compatible(offered: Self, requested: Self) -> bool {
            offered.kind as usize >= requested.kind as usize
        }
    }
    impl Default for Liveliness {
        fn default() -> Self {
            Self {
                kind: LivelinessQosKind::Automatic,
                lease_duration: Duration::INFINITE,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
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

        /// offered is Publisher side QoS value
        /// requested is Subscriber side QoS value
        pub(crate) fn is_compatible(offered: Self, requested: Self) -> bool {
            offered.kind as usize >= requested.kind as usize
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
    impl DestinationOrder {
        /// offered is Publisher side QoS value
        /// requested is Subscriber side QoS value
        pub(crate) fn is_compatible(offered: Self, requested: Self) -> bool {
            offered as usize >= requested as usize
        }
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

    #[derive(Clone, Debug, Serialize, Deserialize)]
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
    impl Default for Partition {
        fn default() -> Self {
            Self {
                name: vec![String::new()],
            }
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

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct WriterDataLifecycle {
        pub autodispose_unregistered_instance: bool,
    }
    impl Default for WriterDataLifecycle {
        fn default() -> Self {
            Self {
                autodispose_unregistered_instance: true,
            }
        }
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

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct TransportPriority {
        pub value: i32,
    }
    #[allow(clippy::derivable_impls)]
    impl Default for TransportPriority {
        fn default() -> Self {
            Self { value: 0 }
        }
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct EntityFactory {
        pub autoenable_created_entites: bool,
    }
    impl Default for EntityFactory {
        fn default() -> Self {
            Self {
                autoenable_created_entites: true,
            }
        }
    }
}

#[allow(unused_imports)]
mod test {
    use super::policy;
    use crate::structure::Duration;
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
