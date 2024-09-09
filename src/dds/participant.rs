use crate::discovery::{
    discovery_db::DiscoveryDB,
    structure::data::{DiscoveredReaderData, DiscoveredWriterData},
    Discovery,
};
use crate::network::net_util::*;
use crate::rtps::reader::ReaderIngredients;
use crate::rtps::writer::WriterIngredients;
use crate::structure::RTPSEntity;
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
    },
    network::udp_listinig_socket::*,
    structure::{EntityId, EntityKind, GuidPrefix, TopicKind, GUID},
};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::net::Ipv4Addr;
use core::sync::atomic::{AtomicU32, Ordering};
use mio_extras::channel as mio_channel;
use mio_v06::net::UdpSocket;
use rand::rngs::SmallRng;
use std::sync::{Mutex, RwLock};
use std::thread::{self, Builder};

/// DDS DomainParticipant
///
/// factory for the Publisher, Subscriber and Topic.
#[derive(Clone)]
pub struct DomainParticipant {
    inner: Arc<Mutex<DomainParticipantDisc>>,
}

impl RTPSEntity for DomainParticipant {
    fn guid(&self) -> GUID {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .guid()
    }
}

impl DomainParticipant {
    pub fn new(domain_id: u16, small_rng: &mut SmallRng) -> Self {
        let (disc_thread_sender, disc_thread_receiver) =
            mio_channel::channel::<thread::JoinHandle<()>>();
        let (discdb_update_sender, discdb_update_receiver) = mio_channel::channel::<GuidPrefix>();
        let discovery_db = DiscoveryDB::new();
        let (writer_add_sender, writer_add_receiver) =
            mio_channel::channel::<(EntityId, DiscoveredWriterData)>();
        let (reader_add_sender, reader_add_receiver) =
            mio_channel::channel::<(EntityId, DiscoveredReaderData)>();
        let dp = Self {
            inner: Arc::new(Mutex::new(DomainParticipantDisc::new(
                domain_id,
                disc_thread_receiver,
                discovery_db.clone(),
                discdb_update_receiver,
                writer_add_sender,
                reader_add_sender,
                small_rng,
            ))),
        };
        let dp_clone = dp.clone();
        let discovery_handler = Builder::new()
            .name(String::from("discovery"))
            .spawn(|| {
                let mut discovery = Discovery::new(
                    dp_clone,
                    discovery_db,
                    discdb_update_sender,
                    writer_add_receiver,
                    reader_add_receiver,
                );
                discovery.discovery_loop();
            })
            .expect("couldn't spawn discovery thread");
        disc_thread_sender
            .send(discovery_handler)
            .expect("couldn't send channel 'disc_thread_sender'");
        dp
    }
    pub fn create_publisher(&self, qos: PublisherQos) -> Publisher {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .create_publisher(self.clone(), qos)
    }
    pub fn create_subscriber(&self, qos: SubscriberQos) -> Subscriber {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .create_subscriber(self.clone(), qos)
    }
    pub fn create_topic(
        &self,
        name: String,
        type_desc: String,
        kind: TopicKind,
        qos: TopicQos,
    ) -> Topic {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .create_topic(self.clone(), name, type_desc, kind, qos)
    }
    pub fn domain_id(&self) -> u16 {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .domain_id()
    }
    pub fn participant_id(&self) -> u16 {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .participant_id()
    }
    pub(crate) fn gen_entity_key(&self) -> [u8; 3] {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .gen_entity_key()
    }
    pub fn get_default_publisher_qos(&self) -> PublisherQosPolicies {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .get_default_publisher_qos()
    }
    pub fn set_default_publisher_qos(&mut self, qos: PublisherQosPolicies) {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .set_default_publisher_qos(qos);
    }
    pub fn get_default_subscriber_qos(&self) -> SubscriberQosPolicies {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .get_default_subscriber_qos()
    }
    pub fn set_default_subscriber_qos(&mut self, qos: SubscriberQosPolicies) {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .set_default_subscriber_qos(qos);
    }
    pub fn get_default_topic_qos(&self) -> TopicQosPolicies {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .get_default_topic_qos()
    }
    pub fn set_default_topic_qos(&mut self, qos: TopicQosPolicies) {
        self.inner
            .lock()
            .expect("couldn't lock DomainParticipantDisc")
            .set_default_topic_qos(qos);
    }
}

struct DomainParticipantDisc {
    inner: Arc<RwLock<DomainParticipantInner>>,
    disc_thread_receiver: mio_channel::Receiver<thread::JoinHandle<()>>,
}

impl DomainParticipantDisc {
    fn new(
        domain_id: u16,
        disc_thread_receiver: mio_channel::Receiver<thread::JoinHandle<()>>,
        discovery_db: DiscoveryDB,
        discdb_update_receiver: mio_channel::Receiver<GuidPrefix>,
        writer_add_sender: mio_channel::Sender<(EntityId, DiscoveredWriterData)>,
        reader_add_sender: mio_channel::Sender<(EntityId, DiscoveredReaderData)>,
        small_rng: &mut SmallRng,
    ) -> Self {
        Self {
            inner: Arc::new(RwLock::new(DomainParticipantInner::new(
                domain_id,
                discovery_db,
                discdb_update_receiver,
                writer_add_sender,
                reader_add_sender,
                small_rng,
            ))),
            disc_thread_receiver,
        }
    }
    pub fn create_publisher(&self, dp: DomainParticipant, qos: PublisherQos) -> Publisher {
        self.inner
            .read()
            .expect("couldn't read lock DomainParticipantInnet")
            .create_publisher(dp, qos)
    }
    pub fn create_subscriber(&self, dp: DomainParticipant, qos: SubscriberQos) -> Subscriber {
        self.inner
            .read()
            .expect("couldn't read lock DomainParticipantInnet")
            .create_subscriber(dp, qos)
    }
    pub fn create_topic(
        &self,
        dp: DomainParticipant,
        name: String,
        type_desc: String,
        kind: TopicKind,
        qos: TopicQos,
    ) -> Topic {
        self.inner
            .read()
            .expect("couldn't read lock DomainParticipantInnet")
            .create_topic(dp, name, type_desc, kind, qos)
    }
    pub fn domain_id(&self) -> u16 {
        self.inner
            .read()
            .expect("couldn't read lock DomainParticipantInnet")
            .domain_id
    }
    pub fn participant_id(&self) -> u16 {
        self.inner
            .read()
            .expect("couldn't read lock DomainParticipantInnet")
            .participant_id
    }
    pub fn gen_entity_key(&self) -> [u8; 3] {
        self.inner
            .read()
            .expect("couldn't read lock DomainParticipantInnet")
            .gen_entity_key()
    }
    pub fn get_default_publisher_qos(&self) -> PublisherQosPolicies {
        self.inner
            .read()
            .expect("couldn't read lock DomainParticipantInnet")
            .get_default_publisher_qos()
    }
    pub fn set_default_publisher_qos(&mut self, qos: PublisherQosPolicies) {
        self.inner
            .write()
            .expect("couldn't write lock DomainParticipantInnet")
            .set_default_publisher_qos(qos);
    }
    pub fn get_default_subscriber_qos(&self) -> SubscriberQosPolicies {
        self.inner
            .read()
            .expect("couldn't read lock DomainParticipantInnet")
            .get_default_subscriber_qos()
    }
    pub fn set_default_subscriber_qos(&mut self, qos: SubscriberQosPolicies) {
        self.inner
            .write()
            .expect("couldn't write lock DomainParticipantInnet")
            .set_default_subscriber_qos(qos);
    }
    pub fn get_default_topic_qos(&self) -> TopicQosPolicies {
        self.inner
            .read()
            .expect("couldn't read lock DomainParticipantInnet")
            .get_default_topic_qos()
    }
    pub fn set_default_topic_qos(&mut self, qos: TopicQosPolicies) {
        self.inner
            .write()
            .expect("couldn't write lock DomainParticipantInnet")
            .set_default_topic_qos(qos);
    }
}
impl Drop for DomainParticipantDisc {
    fn drop(&mut self) {
        if let Ok(djh) = self.disc_thread_receiver.try_recv() {
            djh.join().unwrap();
        }
    }
}

impl RTPSEntity for DomainParticipantDisc {
    fn guid(&self) -> GUID {
        self.inner
            .read()
            .expect("couldn't read lock DomainParticipantInnet")
            .my_guid
    }
}

struct DomainParticipantInner {
    domain_id: u16,
    participant_id: u16,
    pub my_guid: GUID,
    add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
    add_reader_sender: mio_channel::SyncSender<ReaderIngredients>,
    ev_loop_handler: Option<thread::JoinHandle<()>>,
    entity_key_generator: AtomicU32,
    default_publisher_qos: PublisherQosPolicies,
    default_subscriber_qos: SubscriberQosPolicies,
    default_topic_qos: TopicQosPolicies,
}

impl DomainParticipantInner {
    pub fn new(
        domain_id: u16,
        discovery_db: DiscoveryDB,
        discdb_update_receiver: mio_channel::Receiver<GuidPrefix>,
        writer_add_sender: mio_channel::Sender<(EntityId, DiscoveredWriterData)>,
        reader_add_sender: mio_channel::Sender<(EntityId, DiscoveredReaderData)>,
        small_rng: &mut SmallRng,
    ) -> DomainParticipantInner {
        let mut socket_list: BTreeMap<mio_v06::Token, UdpSocket> = BTreeMap::new();
        let spdp_multi_socket = new_multicast(
            "0.0.0.0",
            spdp_multicast_port(domain_id),
            Ipv4Addr::new(239, 255, 0, 1),
        );
        let discovery_multi = match spdp_multi_socket {
            Ok(s) => s,
            Err(e) => panic!("{:?}", e),
        };
        let usertraffic_multi_socket = new_multicast(
            "0.0.0.0",
            usertraffic_multicast_port(domain_id),
            Ipv4Addr::new(239, 255, 0, 1),
        );
        let usertraffic_multi = match usertraffic_multi_socket {
            Ok(s) => s,
            Err(e) => panic!("{:?}", e),
        };

        let mut participant_id = 0;
        let mut discovery_uni: Option<UdpSocket> = None;
        while discovery_uni.is_none() && participant_id < 120 {
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

        socket_list.insert(DISCOVERY_UNI_TOKEN, discovery_multi);
        socket_list.insert(DISCOVERY_MULTI_TOKEN, discovery_uni);
        socket_list.insert(USERTRAFFIC_UNI_TOKEN, usertraffic_uni);
        socket_list.insert(USERTRAFFIC_MULTI_TOKEN, usertraffic_multi);

        let (add_writer_sender, add_writer_receiver) =
            mio_channel::sync_channel::<WriterIngredients>(10);
        let (add_reader_sender, add_reader_receiver) =
            mio_channel::sync_channel::<ReaderIngredients>(10);

        let my_guid = GUID::new_participant_guid(small_rng);

        let ev_loop_handler = thread::Builder::new()
            .name("EventLoop".to_string())
            .spawn(move || {
                let guid_prefix = my_guid.guid_prefix;
                let ev_loop = EventLoop::new(
                    domain_id,
                    socket_list,
                    guid_prefix,
                    add_writer_receiver,
                    add_reader_receiver,
                    writer_add_sender,
                    reader_add_sender,
                    discovery_db,
                    discdb_update_receiver,
                );
                ev_loop.event_loop();
            })
            .expect("couldn't spawn EventLoop thread");
        let default_topic_qos = TopicQosBuilder::new().build();
        let default_publisher_qos = PublisherQosBuilder::new().build();
        let default_subscriber_qos = SubscriberQosBuilder::new().build();

        Self {
            domain_id,
            participant_id: 0,
            my_guid,
            add_writer_sender,
            add_reader_sender,
            ev_loop_handler: Some(ev_loop_handler),
            entity_key_generator: AtomicU32::new(0x0300),
            default_publisher_qos,
            default_subscriber_qos,
            default_topic_qos,
        }
    }

    fn create_publisher(&self, dp: DomainParticipant, qos: PublisherQos) -> Publisher {
        // add_writer用のチャネルを生やして、senderはpubにreceiverは自分
        let guid = GUID::new(
            self.my_guid.guid_prefix,
            EntityId::new_with_entity_kind(self.gen_entity_key(), EntityKind::PUBLISHER),
        );
        match qos {
            PublisherQos::Default => Publisher::new(
                guid,
                self.default_publisher_qos.clone(),
                dp,
                self.add_writer_sender.clone(),
            ),
            PublisherQos::Policies(q) => {
                Publisher::new(guid, q, dp, self.add_writer_sender.clone())
            }
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
                self.add_reader_sender.clone(),
            ),
            SubscriberQos::Policies(q) => {
                Subscriber::new(guid, q, dp, self.add_reader_sender.clone())
            }
        }
    }

    fn create_topic(
        &self,
        dp: DomainParticipant,
        name: String,
        type_desc: String,
        kind: TopicKind,
        qos: TopicQos,
    ) -> Topic {
        match qos {
            TopicQos::Default => {
                Topic::new(name, type_desc, dp, self.default_topic_qos.clone(), kind)
            }
            TopicQos::Policies(q) => Topic::new(name, type_desc, dp, q, kind),
        }
    }

    pub fn gen_entity_key(&self) -> [u8; 3] {
        // entity_keyはGUID Prefixを共有するentityの中で一意であればよい
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
    }
}
