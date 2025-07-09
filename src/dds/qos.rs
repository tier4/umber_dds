//! set of DDS QoS policies for each Entity and its builder

// DDS 1.4 spec: 2.3.3 DCPS PSM : IDL
// How to impl builder: https://keens.github.io/blog/2017/02/09/rustnochottoyarisuginabuilderpata_n/

use policy::*;

macro_rules! getter_method {
    ($name:ident, $policy_type:ident) => {
        pub fn $name(&self) -> $policy_type {
            self.$name.clone()
        }
    };
}

macro_rules! getter_method_with_bool {
    ($name:ident, $policy_type:ident) => {
        pub fn $name(&self) -> $policy_type {
            self.$name.0.clone()
        }
    };
}

/// for setting QoS on a DomainParticipant
#[derive(Clone)]
pub enum DomainParticipantQos {
    /// represent default QoS of DomainParticipant.
    Default,
    Policies(DomainParticipantQosPolicies),
}

/// A collection of QoS policies for configuring the behavior of a DomainParticipant
#[derive(Clone)]
pub struct DomainParticipantQosPolicies {
    user_data: UserData,
    entity_factory: EntityFactory,
}

impl DomainParticipantQosPolicies {
    getter_method!(user_data, UserData);
    getter_method!(entity_factory, EntityFactory);
}

/// for setting QoS on a Topic
#[derive(Clone)]
pub enum TopicQos {
    /// represent default QoS of Topic.
    ///
    /// it can get `DomainParticipant::get_defaul_topict_qos()` and
    /// change `DomainParticipant::set_default_topic_qos()`
    Default,
    Policies(Box<TopicQosPolicies>),
}

/// A collection of QoS policies for configuring the behavior of a Topic
///
/// Each member of TopicQosPolicies has the type (policy_type, bool).
/// The bool flag indicates whether the policy was explicitly set by the user.
#[derive(Clone)]
pub struct TopicQosPolicies {
    topic_data: (TopicData, bool),
    durability: (Durability, bool),
    durability_service: (DurabilityService, bool),
    deadline: (Deadline, bool),
    latency_budget: (LatencyBudget, bool),
    liveliness: (Liveliness, bool),
    reliability: (Reliability, bool),
    destination_order: (DestinationOrder, bool),
    history: (History, bool),
    resource_limits: (ResourceLimits, bool),
    transport_priority: (TransportPriority, bool),
    lifespan: (Lifespan, bool),
    ownership: (Ownership, bool),
}
impl TopicQosPolicies {
    getter_method_with_bool!(topic_data, TopicData);
    getter_method_with_bool!(durability, Durability);
    getter_method_with_bool!(durability_service, DurabilityService);
    getter_method_with_bool!(deadline, Deadline);
    getter_method_with_bool!(latency_budget, LatencyBudget);
    getter_method_with_bool!(liveliness, Liveliness);
    getter_method_with_bool!(reliability, Reliability);
    getter_method_with_bool!(destination_order, DestinationOrder);
    getter_method_with_bool!(history, History);
    getter_method_with_bool!(resource_limits, ResourceLimits);
    getter_method_with_bool!(transport_priority, TransportPriority);
    getter_method_with_bool!(lifespan, Lifespan);
    getter_method_with_bool!(ownership, Ownership);

    pub fn to_datawriter_qos(&self) -> DataWriterQosPolicies {
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
            user_data: (UserData::default(), false),
            ownership: self.ownership,
            ownership_strength: (OwnershipStrength::default(), false),
            writer_data_lifecycle: (WriterDataLifecycle::default(), false),
        }
    }
    pub fn to_datareader_qos(&self) -> DataReaderQosPolicies {
        DataReaderQosPolicies {
            durability: self.durability,
            deadline: self.deadline,
            latency_budget: self.latency_budget,
            liveliness: self.liveliness,
            reliability: self.reliability,
            destination_order: self.destination_order,
            history: self.history,
            resource_limits: self.resource_limits,
            user_data: (UserData::default(), false),
            ownership: self.ownership,
            time_based_filter: (TimeBasedFilter::default(), false),
            reader_data_lifecycle: (ReaderDataLifecycle::default(), false),
        }
    }
}

/// for setting QoS on a DataWriter
#[derive(Clone)]
pub enum DataWriterQos {
    /// represent default QoS of DataWriter.
    ///
    /// it can get `Publisher::get_default_datawriter_qos()` and
    /// change `Publisher::set_default_datawriter_qos()`
    Default,
    Policies(Box<DataWriterQosPolicies>),
}

/// A collection of QoS policies for configuring the behavior of a DataWriter
///
/// Each member of DataWriterQosPolicies has the type (policy_type, bool).
/// The bool flag indicates whether the policy was explicitly set by the user.
#[derive(Clone, PartialEq)]
pub struct DataWriterQosPolicies {
    durability: (Durability, bool),
    durability_service: (DurabilityService, bool),
    deadline: (Deadline, bool),
    latency_budget: (LatencyBudget, bool),
    liveliness: (Liveliness, bool),
    reliability: (Reliability, bool),
    destination_order: (DestinationOrder, bool),
    history: (History, bool),
    resource_limits: (ResourceLimits, bool),
    transport_priority: (TransportPriority, bool),
    lifespan: (Lifespan, bool),
    user_data: (UserData, bool),
    ownership: (Ownership, bool),
    ownership_strength: (OwnershipStrength, bool),
    writer_data_lifecycle: (WriterDataLifecycle, bool),
}

impl DataWriterQosPolicies {
    getter_method_with_bool!(durability, Durability);
    getter_method_with_bool!(durability_service, DurabilityService);
    getter_method_with_bool!(deadline, Deadline);
    getter_method_with_bool!(latency_budget, LatencyBudget);
    getter_method_with_bool!(liveliness, Liveliness);
    getter_method_with_bool!(reliability, Reliability);
    getter_method_with_bool!(destination_order, DestinationOrder);
    getter_method_with_bool!(history, History);
    getter_method_with_bool!(resource_limits, ResourceLimits);
    getter_method_with_bool!(transport_priority, TransportPriority);
    getter_method_with_bool!(lifespan, Lifespan);
    getter_method_with_bool!(user_data, UserData);
    getter_method_with_bool!(ownership, Ownership);
    getter_method_with_bool!(ownership_strength, OwnershipStrength);
    getter_method_with_bool!(writer_data_lifecycle, WriterDataLifecycle);

    pub fn is_compatible(&self, qos: &DataReaderQosPolicies) -> Result<(), String> {
        let mut msg = String::from("{ ");
        let mut is_ok = true;
        if !Durability::is_compatible(self.durability.0, qos.durability.0) {
            is_ok = false;
            msg += &format!(
                "{{ durability is not compatible. writer(self): {:?}, reader(remote): {:?} }}, ",
                self.durability, qos.durability
            );
        }
        if !Deadline::is_compatible(self.deadline.0, qos.deadline.0) {
            is_ok = false;
            msg += &format!(
                "{{ deadline is not compatible. writer(self): {:?}, reader(remote): {:?} }}, ",
                self.deadline, qos.deadline
            );
        }
        if !LatencyBudget::is_compatible(self.latency_budget.0, qos.latency_budget.0) {
            is_ok = false;
            msg += &format!(
                "{{ latency_budget is not compatible. writer(self): {:?}, reader(remote): {:?} }}, ",
                self.latency_budget, qos.latency_budget
            );
        }
        if !Ownership::is_compatible(self.ownership.0, qos.ownership.0) {
            is_ok = false;
            msg += &format!(
                "{{ ownership is not compatible. writer(self): {:?}, reader(remote): {:?} }}, ",
                self.ownership, qos.ownership
            );
        }
        if !Liveliness::is_compatible(self.liveliness.0, qos.liveliness.0) {
            is_ok = false;
            msg += &format!(
                "{{ liveliness is not compatible. writer(self): {:?}, reader(remote): {:?} }}, ",
                self.liveliness, qos.liveliness
            );
        }
        if !Reliability::is_compatible(self.reliability.0, qos.reliability.0) {
            is_ok = false;
            msg += &format!(
                "{{ reliability is not compatible. writer(self): {:?}, reader(remote): {:?} }}, ",
                self.reliability, qos.reliability
            );
        }
        if !DestinationOrder::is_compatible(self.destination_order.0, qos.destination_order.0) {
            is_ok = false;
            msg += &format!(
                "{{ destination_order is not compatible. writer(self): {:?}, reader(remote): {:?} }}, ",
                self.destination_order, qos.destination_order
            );
        }
        if is_ok {
            Ok(())
        } else {
            msg.pop();
            msg.pop();
            msg += " }";
            Err(msg)
        }
    }

    pub fn combine(&mut self, policies: Self) {
        macro_rules! combine_policy {
            ($policy_name:ident, $policy_type:ident) => {
                if self.$policy_name.0 != policies.$policy_name.0 {
                    // If the policies differ, select the one explicitly specified by the user.
                    if policies.$policy_name.1 {
                        self.$policy_name = policies.$policy_name;
                    }
                } else {
                    // If the policies are identical and at least one of them was explicitly set by the user, the resulting combined policy is still regarded as user-specified.
                    self.$policy_name.1 |= policies.$policy_name.1
                }
            };
        }
        combine_policy!(durability, Durability);
        combine_policy!(durability_service, DurabilityService);
        combine_policy!(deadline, Deadline);
        combine_policy!(latency_budget, LatencyBudget);
        combine_policy!(liveliness, Liveliness);
        if self.reliability.0 != policies.reliability.0 && policies.reliability.1 {
            self.reliability = policies.reliability;
        }
        combine_policy!(destination_order, DestinationOrder);
        combine_policy!(history, History);
        combine_policy!(resource_limits, ResourceLimits);
        combine_policy!(transport_priority, TransportPriority);
        combine_policy!(lifespan, Lifespan);
        combine_policy!(user_data, UserData);
        combine_policy!(ownership, Ownership);
        combine_policy!(ownership_strength, OwnershipStrength);
        combine_policy!(writer_data_lifecycle, WriterDataLifecycle);
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
    Policies(Box<PublisherQosPolicies>),
}

/// A collection of QoS policies for configuring the behavior of a Publisher
#[derive(Clone)]
pub struct PublisherQosPolicies {
    presentation: Presentation,
    partition: Partition,
    group_data: GroupData,
    entity_factory: EntityFactory,
}

impl PublisherQosPolicies {
    getter_method!(presentation, Presentation);
    getter_method!(partition, Partition);
    getter_method!(group_data, GroupData);
    getter_method!(entity_factory, EntityFactory);
}

/// for setting QoS on a DataReader
#[derive(Clone)]
pub enum DataReaderQos {
    /// represent default QoS of DataReader.
    ///
    /// it can get `Subscriber::get_default_datareader_qos()` and
    /// change `Subscriber::set_default_datareader_qos()`
    Default,
    Policies(Box<DataReaderQosPolicies>),
}

/// A collection of QoS policies for configuring the behavior of a DataReader
///
/// Each member of DataWriterQosPolicies has the type (policy_type, bool).
/// The bool flag indicates whether the policy was explicitly set by the user.
#[derive(Clone, PartialEq)]
pub struct DataReaderQosPolicies {
    durability: (Durability, bool),
    deadline: (Deadline, bool),
    latency_budget: (LatencyBudget, bool),
    liveliness: (Liveliness, bool),
    reliability: (Reliability, bool),
    destination_order: (DestinationOrder, bool),
    history: (History, bool),
    resource_limits: (ResourceLimits, bool),
    user_data: (UserData, bool),
    ownership: (Ownership, bool),
    time_based_filter: (TimeBasedFilter, bool),
    reader_data_lifecycle: (ReaderDataLifecycle, bool),
}
impl DataReaderQosPolicies {
    getter_method_with_bool!(durability, Durability);
    getter_method_with_bool!(deadline, Deadline);
    getter_method_with_bool!(latency_budget, LatencyBudget);
    getter_method_with_bool!(liveliness, Liveliness);
    getter_method_with_bool!(reliability, Reliability);
    getter_method_with_bool!(destination_order, DestinationOrder);
    getter_method_with_bool!(history, History);
    getter_method_with_bool!(resource_limits, ResourceLimits);
    getter_method_with_bool!(user_data, UserData);
    getter_method_with_bool!(ownership, Ownership);
    getter_method_with_bool!(time_based_filter, TimeBasedFilter);
    getter_method_with_bool!(reader_data_lifecycle, ReaderDataLifecycle);

    pub fn is_compatible(&self, qos: &DataWriterQosPolicies) -> Result<(), String> {
        let mut msg = String::from("{ ");
        let mut is_ok = true;
        if !Durability::is_compatible(qos.durability.0, self.durability.0) {
            is_ok = false;
            msg += &format!(
                "{{ durability is not compatible. reader(self): {:?}, writer(remote): {:?} }}, ",
                self.durability, qos.durability
            );
        }
        if !Deadline::is_compatible(qos.deadline.0, self.deadline.0) {
            is_ok = false;
            msg += &format!(
                "{{ deadline is not compatible. reader(self): {:?}, writer(remote): {:?} }}, ",
                self.deadline, qos.deadline
            );
        }
        if !LatencyBudget::is_compatible(qos.latency_budget.0, self.latency_budget.0) {
            is_ok = false;
            msg += &format!(
                "{{ latency_budget is not compatible. reader(self): {:?}, writer(remote): {:?} }}, ",
                self.latency_budget, qos.latency_budget
            );
        }
        if !Ownership::is_compatible(qos.ownership.0, self.ownership.0) {
            is_ok = false;
            msg += &format!(
                "{{ ownership is not compatible. reader(self): {:?}, writer(remote): {:?} }}, ",
                self.ownership, qos.ownership
            );
        }
        if !Liveliness::is_compatible(qos.liveliness.0, self.liveliness.0) {
            is_ok = false;
            msg += &format!(
                "{{ liveliness is not compatible. reader(self): {:?}, writer(remote): {:?} }}, ",
                self.liveliness, qos.liveliness
            );
        }
        if !Reliability::is_compatible(qos.reliability.0, self.reliability.0) {
            is_ok = false;
            msg += &format!(
                "{{ reliability is not compatible. reader(self): {:?}, writer(remote): {:?} }}, ",
                self.reliability, qos.reliability
            );
        }
        if !DestinationOrder::is_compatible(qos.destination_order.0, self.destination_order.0) {
            is_ok = false;
            msg += &format!(
                "{{ destination_order is not compatible. reader(self): {:?}, writer(remote): {:?} }}, ",
                self.destination_order, qos.destination_order
            );
        }
        if is_ok {
            Ok(())
        } else {
            msg.pop();
            msg.pop();
            msg += " }";
            Err(msg)
        }
    }

    pub fn combine(&mut self, policies: Self) {
        macro_rules! combine_policy {
            ($policy_name:ident, $policy_type:ident) => {
                if self.$policy_name.0 != policies.$policy_name.0 {
                    // If the policies differ, select the one explicitly specified by the user.
                    if policies.$policy_name.1 {
                        self.$policy_name = policies.$policy_name;
                    }
                } else {
                    // If the policies are identical and at least one of them was explicitly set by the user, the resulting combined policy is still regarded as user-specified.
                    self.$policy_name.1 |= policies.$policy_name.1
                }
            };
        }
        combine_policy!(durability, Durability);
        combine_policy!(deadline, Deadline);
        combine_policy!(latency_budget, LatencyBudget);
        combine_policy!(liveliness, Liveliness);
        if self.reliability.0 != policies.reliability.0 && policies.reliability.1 {
            self.reliability = policies.reliability;
        }
        combine_policy!(destination_order, DestinationOrder);
        combine_policy!(history, History);
        combine_policy!(resource_limits, ResourceLimits);
        combine_policy!(user_data, UserData);
        combine_policy!(ownership, Ownership);
        combine_policy!(time_based_filter, TimeBasedFilter);
        combine_policy!(reader_data_lifecycle, ReaderDataLifecycle);
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
    Policies(Box<SubscriberQosPolicies>),
}

/// A collection of QoS policies for configuring the behavior of a Subscriber
#[derive(Clone)]
pub struct SubscriberQosPolicies {
    presentation: Presentation,
    partition: Partition,
    group_data: GroupData,
    entity_factory: EntityFactory,
}

impl SubscriberQosPolicies {
    getter_method!(presentation, Presentation);
    getter_method!(partition, Partition);
    getter_method!(group_data, GroupData);
    getter_method!(entity_factory, EntityFactory);
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
        macro_rules! decide_value {
            ($name:ident, $type:ident) => {
                match self.$name {
                    Some(qos_policy) => (qos_policy, true),
                    None => ($type::default(), false),
                }
            };
        }
        TopicQosPolicies {
            topic_data: decide_value!(topic_data, TopicData),
            durability: decide_value!(durability, Durability),
            durability_service: decide_value!(durability_service, DurabilityService),
            deadline: decide_value!(deadline, Deadline),
            latency_budget: decide_value!(latency_budget, LatencyBudget),
            liveliness: decide_value!(liveliness, Liveliness),
            reliability: match self.reliability {
                Some(reliability) => (reliability, true),
                None => (Reliability::default_besteffort(), false),
            },
            destination_order: decide_value!(destination_order, DestinationOrder),
            history: decide_value!(history, History),
            resource_limits: decide_value!(resource_limits, ResourceLimits),
            transport_priority: decide_value!(transport_priority, TransportPriority),
            lifespan: decide_value!(lifespan, Lifespan),
            ownership: decide_value!(ownership, Ownership),
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
        macro_rules! decide_value {
            ($name:ident, $type:ident) => {
                match self.$name {
                    Some(qos_policy) => (qos_policy, true),
                    None => ($type::default(), false),
                }
            };
        }
        DataWriterQosPolicies {
            durability: decide_value!(durability, Durability),
            durability_service: decide_value!(durability_service, DurabilityService),
            deadline: decide_value!(deadline, Deadline),
            latency_budget: decide_value!(latency_budget, LatencyBudget),
            liveliness: decide_value!(liveliness, Liveliness),
            reliability: match self.reliability {
                Some(reliability) => (reliability, true),
                None => (Reliability::default_reliable(), false),
            },
            destination_order: decide_value!(destination_order, DestinationOrder),
            history: decide_value!(history, History),
            resource_limits: decide_value!(resource_limits, ResourceLimits),
            transport_priority: decide_value!(transport_priority, TransportPriority),
            lifespan: decide_value!(lifespan, Lifespan),
            user_data: decide_value!(user_data, UserData),
            ownership: decide_value!(ownership, Ownership),
            ownership_strength: decide_value!(ownership_strength, OwnershipStrength),
            writer_data_lifecycle: decide_value!(writer_data_lifecycle, WriterDataLifecycle),
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

/// Builder of DataReaderQosPolicies
#[derive(Default)]
pub struct DataReaderQosBuilder {
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

impl DataReaderQosBuilder {
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

    pub fn build(self) -> DataReaderQosPolicies {
        macro_rules! decide_value {
            ($name:ident, $type:ident) => {
                match self.$name {
                    Some(qos_policy) => (qos_policy, true),
                    None => ($type::default(), false),
                }
            };
        }
        DataReaderQosPolicies {
            durability: decide_value!(durability, Durability),
            deadline: decide_value!(deadline, Deadline),
            latency_budget: decide_value!(latency_budget, LatencyBudget),
            liveliness: decide_value!(liveliness, Liveliness),
            reliability: match self.reliability {
                Some(reliability) => (reliability, true),
                None => (Reliability::default_besteffort(), false),
            },
            destination_order: decide_value!(destination_order, DestinationOrder),
            history: decide_value!(history, History),
            resource_limits: decide_value!(resource_limits, ResourceLimits),
            user_data: decide_value!(user_data, UserData),
            ownership: decide_value!(ownership, Ownership),
            time_based_filter: decide_value!(time_based_filter, TimeBasedFilter),
            reader_data_lifecycle: decide_value!(reader_data_lifecycle, ReaderDataLifecycle),
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
    #[repr(i32)]
    /// Durability QoS policy
    ///
    /// rtps 2.3 spec, 8.7.2.2 DDS QoS Parameters that affect the wire protoco
    /// 8.7.2.2.1 DURABILITY
    /// > While volatile and transient-local durability do not affect the RTPS protocol,
    /// > support for transient and persistent durability may.
    ///
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
    pub struct Presentation {
        pub access_scope: PresentationQosAccessScopeKind,
        pub coherent_access: bool,
        pub ordered_access: bool,
    }
    impl Presentation {
        /// offered is Publisher side QoS value
        /// requested is Subscriber side QoS value
        pub(crate) fn _is_compatible(offered: Self, requested: Self) -> bool {
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
    pub struct OwnershipStrength(pub i32);
    #[allow(clippy::derivable_impls)]
    impl Default for OwnershipStrength {
        fn default() -> Self {
            Self(0)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
    pub struct Liveliness {
        pub kind: LivelinessQosKind,
        pub lease_duration: Duration,
    }
    impl Liveliness {
        /// offered is Publisher side QoS value
        /// requested is Subscriber side QoS value
        pub(crate) fn is_compatible(offered: Self, requested: Self) -> bool {
            offered.kind as usize >= requested.kind as usize
                && offered.lease_duration <= requested.lease_duration
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
    #[repr(i32)]
    pub enum LivelinessQosKind {
        Automatic = 0,
        ManualByParticipant = 1,
        ManualByTopic = 2,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
    #[repr(i32)]
    pub enum ReliabilityQosKind {
        Reliable = 2,
        BestEffort = 1,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
    #[repr(i32)]
    pub enum HistoryQosKind {
        KeepLast = 0,
        KeepAll = 1,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
    pub struct Lifespan(pub Duration);
    impl Default for Lifespan {
        fn default() -> Self {
            Self(Duration::INFINITE)
        }
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    pub struct UserData {
        pub value: Vec<u8>,
    }
    impl UserData {
        pub fn serialized_size(&self) -> u16 {
            4 + self.value.len() as u16
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    pub struct TopicData {
        pub value: Vec<u8>,
    }
    impl TopicData {
        pub fn serialized_size(&self) -> u16 {
            4 + self.value.len() as u16
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
    pub struct GroupData {
        pub value: Vec<u8>,
    }
    impl GroupData {
        pub fn serialized_size(&self) -> u16 {
            4 + self.value.len() as u16
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
    pub struct TransportPriority {
        pub value: i32,
    }
    #[allow(clippy::derivable_impls)]
    impl Default for TransportPriority {
        fn default() -> Self {
            Self { value: 0 }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod test {
    use super::{policy, DataWriterQosBuilder, TopicQosBuilder};
    use crate::structure::Duration;
    use cdr::{Infinite, PlCdrLe};

    #[test]
    fn test_combine() {
        let topic_qos = TopicQosBuilder::new()
            .durability(policy::Durability::Volatile)
            .liveliness(policy::Liveliness {
                kind: policy::LivelinessQosKind::ManualByTopic,
                lease_duration: Duration::new(10, 0),
            })
            .build();
        assert_eq!(topic_qos.history, (policy::History::default(), false));
        let dw_qos = DataWriterQosBuilder::new()
            .durability(policy::Durability::TransientLocal)
            .history(policy::History {
                kind: policy::HistoryQosKind::KeepLast,
                depth: 1,
            })
            .build();
        assert_eq!(
            dw_qos.history,
            (
                policy::History {
                    kind: policy::HistoryQosKind::KeepLast,
                    depth: 1,
                },
                true
            )
        );
        let mut dw_qos_combined = topic_qos.to_datawriter_qos();
        assert_eq!(dw_qos_combined.history, (policy::History::default(), false));
        dw_qos_combined.combine(dw_qos);
        assert_eq!(
            dw_qos_combined.durability,
            (policy::Durability::TransientLocal, true)
        );
        assert_eq!(
            dw_qos_combined.history,
            (
                policy::History {
                    kind: policy::HistoryQosKind::KeepLast,
                    depth: 1,
                },
                true
            )
        );
        assert_eq!(
            dw_qos_combined.deadline,
            (policy::Deadline::default(), false)
        );
        assert_eq!(
            dw_qos_combined.liveliness,
            (
                policy::Liveliness {
                    kind: policy::LivelinessQosKind::ManualByTopic,
                    lease_duration: Duration::new(10, 0),
                },
                true
            )
        );
    }

    #[test]
    fn test_serialize() {
        let history = policy::History {
            kind: policy::HistoryQosKind::KeepAll,
            depth: 100,
        };
        let serialized = cdr::serialize::<_, _, PlCdrLe>(&history, Infinite).unwrap();
        /*
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
        */
        assert_eq!(
            serialized,
            vec![0x00, 0x3, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00]
        );
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
