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
    fn new() -> Self {
        QosBuilder::default()
    }

    fn durability(mut self, durability: policy::Durability) -> Self {
        self.durability = Some(durability);
        self
    }

    fn presentation(mut self, presentation: policy::Presentation) -> Self {
        self.presentation = Some(presentation);
        self
    }

    fn deadline(mut self, deadline: policy::Deadline) -> Self {
        self.deadline = Some(deadline);
        self
    }

    fn latency_budget(mut self, latency_budget: policy::LatencyBudget) -> Self {
        self.latency_budget = Some(latency_budget);
        self
    }

    fn ownership(mut self, ownership: policy::Ownership) -> Self {
        self.ownership = Some(ownership);
        self
    }

    fn liveliness(mut self, liveliness: policy::Liveliness) -> Self {
        self.liveliness = Some(liveliness);
        self
    }

    fn time_based_filter(mut self, time_based_filter: policy::TimeBasedFilter) -> Self {
        self.time_based_filter = Some(time_based_filter);
        self
    }

    fn reliability(mut self, reliability: policy::Reliability) -> Self {
        self.reliability = Some(reliability);
        self
    }

    fn destination_order(mut self, destination_order: policy::DestinationOrder) -> Self {
        self.destination_order = Some(destination_order);
        self
    }

    fn history(mut self, history: policy::History) -> Self {
        self.history = Some(history);
        self
    }

    fn resource_limits(mut self, resource_limits: policy::ResourceLimits) -> Self {
        self.resource_limits = Some(resource_limits);
        self
    }

    fn lifespan(mut self, lifespan: policy::Lifespan) -> Self {
        self.lifespan = Some(lifespan);
        self
    }

    fn build(self) -> QosPolicies {
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

mod policy {
    use crate::structure::duration::Duration;

    #[derive(Clone, Copy)]
    pub enum Durability {
        Volatile,
        TransientLocal,
        Transient,
        Persistent,
    }

    #[derive(Clone, Copy)]
    pub struct Presentation {
        pub access_scope: PresentationQosAccessScopeKind,
        pub coherent_access: bool,
        pub oerered_access: bool,
    }

    #[derive(Clone, Copy)]
    pub enum PresentationQosAccessScopeKind {
        Instance,
        Topic,
        Group,
    }

    #[derive(Clone, Copy)]
    pub struct Deadline {
        pub period: Duration,
    }

    #[derive(Clone, Copy)]
    pub struct LatencyBudget(pub Duration);

    #[derive(Clone, Copy)]
    pub enum Ownership {
        Shared,
        Exclusive { strength: i64 },
    }
    #[derive(Clone, Copy)]
    pub struct Liveliness {
        pub lease_duration: Duration,
        pub kind: LivelinessQosKind,
    }

    #[derive(Clone, Copy)]
    pub enum LivelinessQosKind {
        Automatic,
        ManualByParticipant,
        ManualByTopic,
    }

    #[derive(Clone, Copy)]
    pub struct TimeBasedFilter {
        pub minimun_separation: Duration,
    }

    #[derive(Clone, Copy)]
    pub struct Reliability {
        pub kind: ReliabilityQosKind,
        pub max_bloking_time: Duration,
    }

    #[derive(Clone, Copy)]
    pub enum ReliabilityQosKind {
        Reliable,
        BestEffort,
    }

    #[derive(Clone, Copy)]
    pub enum DestinationOrder {
        ByReceptionTimestamp,
        BySourceTimestamp,
    }

    #[derive(Clone, Copy)]
    pub struct History {
        pub kind: HistoryQosKind,
        pub depth: i64,
    }

    #[derive(Clone, Copy)]
    pub enum HistoryQosKind {
        KeepLast,
        LeepAll,
    }

    #[derive(Clone, Copy)]
    pub struct ResourceLimits {
        pub max_samples: i64,
        pub max_instance: i64,
        pub max_samples_per_instanse: i64,
    }

    #[derive(Clone, Copy)]
    pub struct Lifespan(pub Duration);
}
