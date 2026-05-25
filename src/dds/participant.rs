use crate::discovery::{
    create_builtin_endpoints,
    discovery_db::DiscoveryDB,
    structure::{
        builtin_endpoint::BuiltinEndpoint,
        data::{DiscoveredReaderData, DiscoveredWriterData, SPDPdiscoveredParticipantData},
    },
    Discovery, DiscoveryDBUpdateNotifier, ParticipantMessageCmd,
};
use crate::message::{
    message_header::ProtocolVersion,
    submessage::element::{Locator, RepresentationIdentifier, SerializedPayload},
};
use crate::network::{net_util::*, udp_sender::UdpSender};
use crate::rtps::reader::ReaderIngredients;
use crate::rtps::writer::WriterIngredients;
use crate::structure::{RTPSEntity, VendorId};
use crate::{
    dds::{
        event_loop::EventLoop,
        publisher::Publisher,
        qos::{
            PublisherQos, PublisherQosBuilder, PublisherQosPolicies, SubscriberQos,
            SubscriberQosBuilder, SubscriberQosPolicies, TopicQos, TopicQosBuilder,
            TopicQosPolicies,
        },
        subscriber::Subscriber,
        tokens::*,
        topic::Topic,
        DdsData,
    },
    network::udp_listinig_socket::*,
    structure::{EntityId, EntityKind, TopicKind, GUID},
};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::net::Ipv4Addr;
use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration as CoreDuration;
use enumflags2::make_bitflags;
use log::info;
use mio_extras::channel as mio_channel;
use mio_v06::net::UdpSocket;
use rand::rngs::SmallRng;
use std::thread::{self, Builder};

use awkernel_sync::{mcs::MCSNode, mutex::Mutex};

/// DDS DomainParticipant
///
/// factory for the Publisher, Subscriber and Topic.
#[derive(Clone)]
pub struct DomainParticipant {
    inner: Arc<Mutex<DomainParticipantInner>>,
}

impl RTPSEntity for DomainParticipant {
    fn guid(&self) -> GUID {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).my_guid
    }
}

struct EvLoopIngredients {
    socket_list: BTreeMap<mio_v06::Token, UdpSocket>,
    udp_sender: UdpSender,
    create_writer_receiver: mio_extras::channel::Receiver<WriterIngredients>,
    create_reader_receiver: mio_extras::channel::Receiver<ReaderIngredients>,
}

impl DomainParticipant {
    /// + network_interfaces: network interfaces to use sending or receiving message.
    /// + config: If `config` is `None`, the default `ParticipantConfig` will be used.
    ///
    /// By default, Umber DDS selects one non-loopback network interface that has an IPv4 address assigned for sending and receiving messages.
    /// If no such interface exists, the loopback interface is used instead.
    /// When several interfaces are present, Umber DDS obtains the list of interfaces from the operating system,
    /// filters out loopback devices and those without an IPv4 address,
    /// and then chooses the first remaining entry in that list.
    /// If this automatic choice is undesirable for example,
    /// if a local bridge that cannot reach other hosts is selected you can explicitly specify the interface(s) with network_interfaces.
    ///
    /// If you need Umber DDS to operate over multiple interfaces, pass the full set of interfaces you want to use.
    pub fn new(
        domain_id: u16,
        network_interfaces: Vec<Ipv4Addr>,
        config: Option<ParticipantConfig>,
        small_rng: &mut SmallRng,
    ) -> Self {
        let (discdb_update_sender, discdb_update_receiver) =
            mio_channel::channel::<DiscoveryDBUpdateNotifier>();
        let discovery_db = DiscoveryDB::new();
        let (notify_new_writer_sender, notify_new_writer_receiver) =
            mio_channel::channel::<(EntityId, DiscoveredWriterData)>();
        let (notify_new_reader_sender, notify_new_reader_receiver) =
            mio_channel::channel::<(EntityId, DiscoveredReaderData)>();
        let (participant_msg_cmd_sender, participant_msg_cmd_receiver) =
            mio_channel::sync_channel::<ParticipantMessageCmd>(32);

        let participant_config = config.unwrap_or_default();

        let dp_network_interfaces = if network_interfaces.is_empty() {
            let local_ipv4_nics: Vec<Ipv4Addr> = get_local_interfaces()
                .iter()
                .filter_map(|n| match n {
                    std::net::IpAddr::V4(a) => Some(*a),
                    std::net::IpAddr::V6(_) => None,
                })
                .collect();
            if local_ipv4_nics.is_empty() {
                panic!("failed to get local network_interfaces");
            }
            vec![local_ipv4_nics[0]]
        } else {
            network_interfaces
        };

        let (dp_inner, ev_loop_ing) = DomainParticipantInner::new(
            domain_id,
            participant_msg_cmd_sender,
            dp_network_interfaces.clone(),
            participant_config,
            small_rng,
        );
        let dp = Self {
            inner: Arc::new(Mutex::new(dp_inner)),
        };
        let (be, be_ing) = create_builtin_endpoints(&dp);
        let mut node = MCSNode::new();
        let serialized_spdp_data = dp.inner.lock(&mut node).serialized_spdp_data.clone();

        let serialized_spdp_data_clone = serialized_spdp_data.clone();
        let dp_clone = dp.clone();
        let discovery_db_clone = discovery_db.clone();
        let ev_loop_handler = thread::Builder::new()
            .name("EventLoop".to_string())
            .spawn(move || {
                let guid_prefix = dp_clone.guid_prefix();
                let ev_loop = EventLoop::new(
                    domain_id,
                    guid_prefix,
                    ev_loop_ing.socket_list,
                    ev_loop_ing.udp_sender,
                    guid_prefix,
                    ev_loop_ing.create_writer_receiver,
                    ev_loop_ing.create_reader_receiver,
                    notify_new_writer_sender,
                    notify_new_reader_sender,
                    discovery_db_clone,
                    discdb_update_receiver,
                    serialized_spdp_data_clone,
                    be_ing,
                );
                ev_loop.event_loop();
            })
            .expect("failed to spawn EventLoop thread");
        let mut node = MCSNode::new();
        dp.inner.lock(&mut node).ev_loop_handler = Some(ev_loop_handler);

        let dp_clone = dp.clone();
        let discovery_handler = Builder::new()
            .name(String::from("discovery"))
            .spawn(|| {
                let mut discovery = Discovery::new(
                    dp_clone,
                    be,
                    discovery_db,
                    serialized_spdp_data,
                    discdb_update_sender,
                    notify_new_writer_receiver,
                    notify_new_reader_receiver,
                    participant_msg_cmd_receiver,
                );
                discovery.discovery_loop();
            })
            .expect("failed to spawn discovery thread");
        let mut node = MCSNode::new();
        dp.inner.lock(&mut node).discovery_handler = Some(discovery_handler);

        info!("created new Participant {}", dp.guid());
        dp
    }
    pub fn create_publisher(&self, qos: PublisherQos) -> Publisher {
        let mut node = MCSNode::new();
        self.inner
            .lock(&mut node)
            .create_publisher(self.clone(), qos)
    }
    pub fn create_subscriber(&self, qos: SubscriberQos) -> Subscriber {
        let mut node = MCSNode::new();
        self.inner
            .lock(&mut node)
            .create_subscriber(self.clone(), qos)
    }
    pub fn create_topic<D: DdsData>(&self, name: String, qos: TopicQos) -> Topic {
        let mut node = MCSNode::new();
        self.inner
            .lock(&mut node)
            .create_topic::<D>(self.clone(), name, qos)
    }
    pub(crate) fn create_builtin_topic(
        &self,
        name: String,
        type_desc: String,
        kind: TopicKind,
        qos: TopicQos,
    ) -> Topic {
        let mut node = MCSNode::new();
        self.inner
            .lock(&mut node)
            .create_builtin_topic(self.clone(), name, type_desc, kind, qos)
    }
    pub(crate) fn get_network_interfaces(&self) -> Vec<Ipv4Addr> {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).get_network_interfaces()
    }
    pub fn domain_id(&self) -> u16 {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).domain_id
    }
    pub fn participant_id(&self) -> u16 {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).participant_id
    }
    pub(crate) fn gen_entity_key(&self) -> [u8; 3] {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).gen_entity_key()
    }
    pub(crate) fn get_config(&self) -> ParticipantConfig {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).get_config()
    }
    pub fn get_default_publisher_qos(&self) -> PublisherQosPolicies {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).get_default_publisher_qos()
    }
    pub fn set_default_publisher_qos(&mut self, qos: PublisherQosPolicies) {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).set_default_publisher_qos(qos);
    }
    pub fn get_default_subscriber_qos(&self) -> SubscriberQosPolicies {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).get_default_subscriber_qos()
    }
    pub fn set_default_subscriber_qos(&mut self, qos: SubscriberQosPolicies) {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).set_default_subscriber_qos(qos);
    }
    pub fn get_default_topic_qos(&self) -> TopicQosPolicies {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).get_default_topic_qos()
    }
    pub fn set_default_topic_qos(&mut self, qos: TopicQosPolicies) {
        let mut node = MCSNode::new();
        self.inner.lock(&mut node).set_default_topic_qos(qos);
    }
}

pub(crate) struct DomainParticipantInner {
    domain_id: u16,
    participant_id: u16,
    pub my_guid: GUID,
    create_writer_sender: mio_channel::SyncSender<WriterIngredients>,
    create_reader_sender: mio_channel::SyncSender<ReaderIngredients>,
    ev_loop_handler: Option<thread::JoinHandle<()>>,
    discovery_handler: Option<thread::JoinHandle<()>>,
    entity_key_generator: AtomicU32,
    default_publisher_qos: PublisherQosPolicies,
    default_subscriber_qos: SubscriberQosPolicies,
    default_topic_qos: TopicQosPolicies,
    participant_msg_cmd_sender: mio_channel::SyncSender<ParticipantMessageCmd>,
    participant_config: ParticipantConfig,
    network_interfaces: Vec<Ipv4Addr>,
    _spdp_data: SPDPdiscoveredParticipantData,
    serialized_spdp_data: SerializedPayload,
}

impl DomainParticipantInner {
    #[allow(clippy::too_many_arguments)]
    fn new(
        domain_id: u16,
        participant_msg_cmd_sender: mio_channel::SyncSender<ParticipantMessageCmd>,
        network_interfaces: Vec<Ipv4Addr>,
        participant_config: ParticipantConfig,
        small_rng: &mut SmallRng,
    ) -> (DomainParticipantInner, EvLoopIngredients) {
        let mut socket_list: BTreeMap<mio_v06::Token, UdpSocket> = BTreeMap::new();
        let spdp_multi_socket = new_multicast(
            "0.0.0.0",
            spdp_multicast_port(domain_id),
            &network_interfaces,
            Ipv4Addr::new(239, 255, 0, 1),
        );
        let discovery_multi = match spdp_multi_socket {
            Ok(s) => s,
            Err(e) => panic!("{e:?}"),
        };
        let usertraffic_multi_socket = new_multicast(
            "0.0.0.0",
            usertraffic_multicast_port(domain_id),
            &network_interfaces,
            Ipv4Addr::new(239, 255, 0, 1),
        );
        let usertraffic_multi = match usertraffic_multi_socket {
            Ok(s) => s,
            Err(e) => panic!("{e:?}"),
        };

        // rtps 2.3 spec, 9.6.1.1
        // The domainId and participantId identifiers are used to avoid port conflicts among Participants on the same node.
        // Each Participant on the same node and in the same domain must use a unique participantId. In the case of multicast,
        // all Participants in the same domain share the same port number, so the participantId identifier is not used in the port number expression.
        let mut participant_id = 0;
        let mut discovery_uni: Option<UdpSocket> = None;
        while discovery_uni.is_none() && participant_id < 120 {
            // To simplify the configuration of the SPDP, participantId values ideally start at 0 and are incremented
            // for each additional Participant on the same node and in the same domain.
            match new_unicast("0.0.0.0", spdp_unicast_port(domain_id, participant_id)) {
                Ok(s) => discovery_uni = Some(s),
                Err(_e) => participant_id += 1,
            }
        }

        let discovery_uni = discovery_uni
            .expect("the max number of participant on same host on same domin is 127.");

        let usertraffic_uni = new_unicast(
            "0.0.0.0",
            usertraffic_unicast_port(domain_id, participant_id),
        )
        .expect("the max number of participant on same host on same domin is 127.");

        socket_list.insert(DISCOVERY_UNI_TOKEN, discovery_uni);
        socket_list.insert(DISCOVERY_MULTI_TOKEN, discovery_multi);
        socket_list.insert(USERTRAFFIC_UNI_TOKEN, usertraffic_uni);
        socket_list.insert(USERTRAFFIC_MULTI_TOKEN, usertraffic_multi);

        let (create_writer_sender, create_writer_receiver) =
            mio_channel::sync_channel::<WriterIngredients>(10);
        let (create_reader_sender, create_reader_receiver) =
            mio_channel::sync_channel::<ReaderIngredients>(10);

        let my_guid = GUID::new_participant_guid(small_rng);

        let udp_sender =
            UdpSender::new(0, network_interfaces.clone()).expect("failed to gen UdpSender");

        let spdp_data = SPDPdiscoveredParticipantData::new(
            domain_id,
            String::from("todo"),
            ProtocolVersion::PROTOCOLVERSION,
            my_guid,
            VendorId::THIS_IMPLEMENTATION,
            false,
            make_bitflags!(BuiltinEndpoint::{DISC_BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER}),
            Locator::new_list_from_multi_ipv4(
                spdp_unicast_port(domain_id, participant_id) as u32,
                network_interfaces.clone(),
            ),
            vec![Locator::new_from_ipv4(
                spdp_multicast_port(domain_id) as u32,
                [239, 255, 0, 1],
            )],
            Locator::new_list_from_multi_ipv4(
                usertraffic_unicast_port(domain_id, participant_id) as u32,
                network_interfaces.clone(),
            ),
            vec![Locator::new_from_ipv4(
                usertraffic_multicast_port(domain_id) as u32,
                [239, 255, 0, 1],
            )],
            Some(0),
            participant_config.lease_duration.into(),
        );
        let serialized_spdp_data =
            SerializedPayload::new_from_cdr_data(&spdp_data, RepresentationIdentifier::PL_CDR_LE);

        let default_topic_qos = TopicQosBuilder::new().build();
        let default_publisher_qos = PublisherQosBuilder::new().build();
        let default_subscriber_qos = SubscriberQosBuilder::new().build();

        let dp = Self {
            domain_id,
            participant_id,
            my_guid,
            create_writer_sender,
            create_reader_sender,
            ev_loop_handler: None,
            discovery_handler: None,
            // largest pre-difined entityKey is {00, 02, 01} @DDS-Security 1.1
            // entity_key of user difined entity start {00, 03, 00}
            entity_key_generator: AtomicU32::new(0x0300),
            default_publisher_qos,
            default_subscriber_qos,
            default_topic_qos,
            participant_msg_cmd_sender,
            participant_config,
            network_interfaces,
            _spdp_data: spdp_data,
            serialized_spdp_data,
        };
        let ev_loop_ing = EvLoopIngredients {
            socket_list,
            udp_sender,
            create_writer_receiver,
            create_reader_receiver,
        };
        (dp, ev_loop_ing)
    }

    fn create_publisher(&self, dp: DomainParticipant, qos: PublisherQos) -> Publisher {
        // generate channel for create_writer. publisher hold its sender and, Self hold receiver
        let guid = GUID::new(
            self.my_guid.guid_prefix,
            EntityId::new_with_entity_kind(self.gen_entity_key(), EntityKind::PUBLISHER),
        );
        match qos {
            PublisherQos::Default => Publisher::new(
                guid,
                self.default_publisher_qos.clone(),
                dp,
                self.create_writer_sender.clone(),
                self.participant_msg_cmd_sender.clone(),
            ),
            PublisherQos::Policies(q) => Publisher::new(
                guid,
                *q,
                dp,
                self.create_writer_sender.clone(),
                self.participant_msg_cmd_sender.clone(),
            ),
        }
    }

    fn create_subscriber(&self, dp: DomainParticipant, qos: SubscriberQos) -> Subscriber {
        let guid = GUID::new(
            self.my_guid.guid_prefix,
            EntityId::new_with_entity_kind(self.gen_entity_key(), EntityKind::SUBSCRIBER),
        );
        match qos {
            SubscriberQos::Default => Subscriber::new(
                guid,
                self.default_subscriber_qos.clone(),
                dp,
                self.create_reader_sender.clone(),
            ),
            SubscriberQos::Policies(q) => {
                Subscriber::new(guid, *q, dp, self.create_reader_sender.clone())
            }
        }
    }

    fn create_topic<D: DdsData>(
        &self,
        dp: DomainParticipant,
        name: String,
        qos: TopicQos,
    ) -> Topic {
        match qos {
            TopicQos::Default => Topic::new::<D>(name, dp, self.default_topic_qos.clone()),
            TopicQos::Policies(q) => Topic::new::<D>(name, dp, *q),
        }
    }

    fn create_builtin_topic(
        &self,
        dp: DomainParticipant,
        name: String,
        type_desc: String,
        kind: TopicKind,
        qos: TopicQos,
    ) -> Topic {
        match qos {
            TopicQos::Default => {
                Topic::new_builtin(name, type_desc, dp, kind, self.default_topic_qos.clone())
            }
            TopicQos::Policies(q) => Topic::new_builtin(name, type_desc, dp, kind, *q),
        }
    }

    pub(crate) fn get_network_interfaces(&self) -> Vec<Ipv4Addr> {
        self.network_interfaces.clone()
    }

    pub(crate) fn get_config(&self) -> ParticipantConfig {
        self.participant_config
    }

    pub fn gen_entity_key(&self) -> [u8; 3] {
        // entity_key must unique to participant
        // This implementation use sequential number to entity_key
        // rtps 2.3 spec, 9.3.1.2 Mapping of the EntityId_t
        // When not pre-defined, the entityKey field within the EntityId_t can be chosen arbitrarily
        // by the middleware implementation as long as the resulting EntityId_t
        // is unique within the Participant.
        let [_, a, b, c] = self
            .entity_key_generator
            .fetch_add(1, Ordering::Relaxed)
            .to_be_bytes();
        [a, b, c]
    }

    pub fn get_default_publisher_qos(&self) -> PublisherQosPolicies {
        self.default_publisher_qos.clone()
    }
    pub fn set_default_publisher_qos(&mut self, qos: PublisherQosPolicies) {
        self.default_publisher_qos = qos;
    }
    pub fn get_default_subscriber_qos(&self) -> SubscriberQosPolicies {
        self.default_subscriber_qos.clone()
    }
    pub fn set_default_subscriber_qos(&mut self, qos: SubscriberQosPolicies) {
        self.default_subscriber_qos = qos;
    }
    pub fn get_default_topic_qos(&self) -> TopicQosPolicies {
        self.default_topic_qos.clone()
    }
    pub fn set_default_topic_qos(&mut self, qos: TopicQosPolicies) {
        self.default_topic_qos = qos;
    }
}

impl Drop for DomainParticipantInner {
    fn drop(&mut self) {
        if let Some(handler) = self.ev_loop_handler.take() {
            handler.join().unwrap();
        }
        if let Some(handler) = self.discovery_handler.take() {
            handler.join().unwrap();
        }
    }
}

/// 3 seconds
pub const DEFAULT_PARTICIPANT_MESSAGE_PERIOD: CoreDuration = CoreDuration::from_secs(3);
/// 20 seconds
pub const DEFAULT_LEASE_DURATION: CoreDuration = CoreDuration::from_secs(20);
/// 2 seconds
pub const DEFAULT_HEARTBEAT_PERIOD: CoreDuration = CoreDuration::from_secs(2);
/// 0 seconds
pub const DEFAULT_NACK_RESPONSE_DELAY: CoreDuration = CoreDuration::from_secs(0);
/// 0 seconds
pub const DEFAULT_HEARTBEAT_RESPONSE_DELAY: CoreDuration = CoreDuration::from_secs(0);

/// ParticipantConfig
///
/// A set of parameters for configuring the behavior of a DDS Participant or RTPS protocol.
#[derive(Clone, Copy)]
pub struct ParticipantConfig {
    /// The periodic interval at which the Simple Participant Discovery Protocol (SPDP) messages are sent to announce the presence of this Participant.
    ///
    /// This parameter directly affects the time required for Discovery to complete and the time taken to detect the departure of other Participants (liveliness).
    /// A shorter interval leads to faster Discovery but increases the overhead and load on both the Participant and the network.
    ///
    /// The RTPS v2.3 specification defaults this to 30s, but implementations often use smaller values as default for practical reasons (e.g., Cyclone DDS uses 8s, Fast DDS uses 3s).
    ///
    /// default value of Umber DDS is [`DEFAULT_PARTICIPANT_MESSAGE_PERIOD`]
    pub participant_message_period: CoreDuration,
    /// The leaseDuration for the Participant's liveliness.
    ///
    /// If a Simple Participant Discovery Protocol (SPDP) message is not received from
    /// a remote Participant within this period, the remote Participant is presumed
    /// to have left the network. This value must be greater than the
    /// `participant_message_period` (the announcement interval).
    ///
    /// NOTE: This period governs **Participant** liveliness and is distinct from the
    /// `lease_duration` specified in the Liveliness QoS Policy, which governs
    /// **DataWriter** and **DataReader** liveliness.
    ///
    /// default value of Umber DDS is [`DEFAULT_LEASE_DURATION`]
    pub lease_duration: CoreDuration,
    /// The interval at which the Reliable DataWriter sends Heartbeat messages.
    ///
    /// Heartbeat messages inform matched Reliable DataReaders of the Writer's latest
    /// sequence number and the range of data that requires acknowledgment. A shorter
    /// period improves latency for unacknowledged data but increases network traffic.
    ///
    /// default value of Umber DDS is [`DEFAULT_HEARTBEAT_PERIOD`]
    pub heartbeat_period: CoreDuration,
    /// The time interval a Reliable DataWriter waits after receiving an ACKNACK
    /// message before responding with the requested missing data.
    ///
    /// The RTPS v2.3 specification defaults this to 200ms, but implementations often use smaller values as default for practical reasons (e.g., Cyclone DDS uses 0s, Fast DDS uses 5ms).
    ///
    /// default value of Umber DDS is [`DEFAULT_NACK_RESPONSE_DELAY`]
    pub nack_response_delay: CoreDuration,
    /// The time interval a Reliable DataReader waits after receiving a Heartbeat message
    /// before sending an ACKNACK message.
    ///
    /// The RTPS v2.3 specification defaults this to 500ms, but implementations often use smaller values as default for practical reasons (e.g., Cyclone DDS uses 0s, Fast DDS uses 5ms).
    ///
    /// default value of Umber DDS is [`DEFAULT_HEARTBEAT_RESPONSE_DELAY`]
    pub heartbeat_response_delay: CoreDuration,
}

impl Default for ParticipantConfig {
    fn default() -> Self {
        Self {
            participant_message_period: DEFAULT_PARTICIPANT_MESSAGE_PERIOD,
            lease_duration: DEFAULT_LEASE_DURATION,
            heartbeat_period: DEFAULT_HEARTBEAT_PERIOD,
            nack_response_delay: DEFAULT_NACK_RESPONSE_DELAY,
            heartbeat_response_delay: DEFAULT_HEARTBEAT_RESPONSE_DELAY,
        }
    }
}

/// ParticipantConfigBuilder
///
/// builder of ParticipantConfig
///
/// `CoreDuration` is `core::time::Duration`
pub struct ParticipantConfigBuilder {
    participant_message_period: Option<CoreDuration>,
    lease_duration: Option<CoreDuration>,
    heartbeat_period: Option<CoreDuration>,
    nack_response_delay: Option<CoreDuration>,
    heartbeat_response_delay: Option<CoreDuration>,
}

impl ParticipantConfigBuilder {
    pub fn new() -> Self {
        Self {
            participant_message_period: None,
            lease_duration: None,
            heartbeat_period: None,
            nack_response_delay: None,
            heartbeat_response_delay: None,
        }
    }

    pub fn build(self) -> ParticipantConfig {
        let participant_message_period = self
            .participant_message_period
            .unwrap_or(DEFAULT_PARTICIPANT_MESSAGE_PERIOD);
        let lease_duration = self.lease_duration.unwrap_or(DEFAULT_LEASE_DURATION);
        if participant_message_period >= lease_duration {
            panic!("lease_duration must longer than participant_message_period. lease_duration: {:?}, participant_message_period: {:?}", lease_duration, participant_message_period);
        }
        ParticipantConfig {
            participant_message_period,
            lease_duration,
            heartbeat_period: self.heartbeat_period.unwrap_or(DEFAULT_HEARTBEAT_PERIOD),
            nack_response_delay: self
                .nack_response_delay
                .unwrap_or(DEFAULT_NACK_RESPONSE_DELAY),
            heartbeat_response_delay: self
                .heartbeat_response_delay
                .unwrap_or(DEFAULT_HEARTBEAT_RESPONSE_DELAY),
        }
    }

    pub fn participant_period(mut self, period: CoreDuration) -> Self {
        self.participant_message_period = Some(period);
        self
    }
    pub fn lease_duration(mut self, duration: CoreDuration) -> Self {
        self.lease_duration = Some(duration);
        self
    }
    pub fn heartbeat_period(mut self, period: CoreDuration) -> Self {
        self.heartbeat_period = Some(period);
        self
    }
    pub fn nack_response_delay(mut self, period: CoreDuration) -> Self {
        self.nack_response_delay = Some(period);
        self
    }
    pub fn heartbeat_response_delay(mut self, period: CoreDuration) -> Self {
        self.heartbeat_response_delay = Some(period);
        self
    }
}

impl Default for ParticipantConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
