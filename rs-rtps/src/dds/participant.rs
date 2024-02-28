use crate::discovery::discovery::Discovery;
use crate::network::net_util::*;
use crate::rtps::reader::ReaderIngredients;
use crate::rtps::writer::WriterIngredients;
use crate::structure::entity::RTPSEntity;
use crate::{
    dds::{
        event_loop::EventLoop, publisher::Publisher, qos::QosPolicies, subscriber::Subscriber,
        tokens::*,
    },
    network::udp_listinig_socket::*,
    structure::{
        entity_id::{EntityId, EntityKind},
        guid::*,
    },
};
use mio_extras::channel as mio_channel;
use mio_v06::net::UdpSocket;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Mutex;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::thread::{self, Builder};

#[derive(Clone)]
pub struct DomainParticipant {
    inner: Arc<Mutex<DomainParticipantDisc>>,
}

impl RTPSEntity for DomainParticipant {
    fn guid(&self) -> GUID {
        self.inner.lock().unwrap().guid()
    }
}

impl DomainParticipant {
    pub fn new(domain_id: u16) -> Self {
        let (disc_thread_sender, disc_thread_receiver) =
            mio_channel::channel::<thread::JoinHandle<()>>();
        let dp = Self {
            inner: Arc::new(Mutex::new(DomainParticipantDisc::new(
                domain_id,
                disc_thread_receiver,
            ))),
        };
        let dp_clone = dp.clone();
        let discovery_handler = Builder::new()
            .name(String::from("discovery"))
            .spawn(|| {
                let mut discovery = Discovery::new(dp_clone);
                discovery.discovery_loop();
            })
            .unwrap();
        disc_thread_sender.send(discovery_handler).unwrap();
        dp
    }
    pub fn create_publisher(&self, qos: QosPolicies) -> Publisher {
        self.inner
            .lock()
            .unwrap()
            .create_publisher(self.clone(), qos)
    }
    pub fn create_subscriber(&self, qos: QosPolicies) -> Subscriber {
        self.inner
            .lock()
            .unwrap()
            .create_subscriber(self.clone(), qos)
    }
    pub fn domain_id(&self) -> u16 {
        self.inner.lock().unwrap().domain_id()
    }
    pub fn gen_entity_key(&self) -> [u8; 3] {
        self.inner.lock().unwrap().gen_entity_key()
    }
}

struct DomainParticipantDisc {
    inner: Arc<DomainParticipantInner>,
    disc_thread_receiver: mio_channel::Receiver<thread::JoinHandle<()>>,
}

impl DomainParticipantDisc {
    fn new(
        domain_id: u16,
        disc_thread_receiver: mio_channel::Receiver<thread::JoinHandle<()>>,
    ) -> Self {
        Self {
            inner: Arc::new(DomainParticipantInner::new(domain_id)),
            disc_thread_receiver,
        }
    }
    pub fn create_publisher(&self, dp: DomainParticipant, qos: QosPolicies) -> Publisher {
        self.inner.create_publisher(dp, qos)
    }
    pub fn create_subscriber(&self, dp: DomainParticipant, qos: QosPolicies) -> Subscriber {
        self.inner.create_subscriber(dp, qos)
    }
    pub fn domain_id(&self) -> u16 {
        self.inner.domain_id
    }
    pub fn gen_entity_key(&self) -> [u8; 3] {
        self.inner.gen_entity_key()
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
        self.inner.my_guid
    }
}

struct DomainParticipantInner {
    domain_id: u16,
    participant_id: u16,
    pub my_guid: GUID,
    add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
    add_reader_sender: mio_channel::SyncSender<ReaderIngredients>,
    thread: thread::JoinHandle<()>,
    entity_key_generator: AtomicU32,
}

impl DomainParticipantInner {
    pub fn new(domain_id: u16) -> DomainParticipantInner {
        let mut socket_list: HashMap<mio_v06::Token, UdpSocket> = HashMap::new();
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

        let my_guid = GUID::new_participant_guid();

        let new_thread = thread::spawn(move || {
            let guid_prefix = my_guid.guid_prefix;
            let ev_loop = EventLoop::new(
                socket_list,
                guid_prefix,
                add_writer_receiver,
                add_reader_receiver,
            );
            ev_loop.event_loop();
        });

        Self {
            domain_id,
            participant_id: 0,
            my_guid,
            add_writer_sender,
            add_reader_sender,
            thread: new_thread,
            entity_key_generator: AtomicU32::new(0x0300),
        }
    }

    fn create_publisher(&self, dp: DomainParticipant, qos: QosPolicies) -> Publisher {
        // add_writer用のチャネルを生やして、senderはpubにreceiverは自分
        let guid = GUID::new(
            self.my_guid.guid_prefix,
            EntityId::new_with_entity_kind(self.gen_entity_key(), EntityKind::PUBLISHER),
        );
        Publisher::new(
            guid,
            qos.clone(),
            qos.clone(),
            dp,
            self.add_writer_sender.clone(),
        )
    }

    fn create_subscriber(&self, dp: DomainParticipant, qos: QosPolicies) -> Subscriber {
        let guid = GUID::new(
            self.my_guid.guid_prefix,
            EntityId::new_with_entity_kind(self.gen_entity_key(), EntityKind::SUBSCRIBER),
        );
        Subscriber::new(guid, qos, dp, self.add_reader_sender.clone())
    }

    pub fn gen_entity_key(&self) -> [u8; 3] {
        // entity_keyはGUID Prefixを共有するentityの中で一意であればよい
        let [_, a, b, c] = self
            .entity_key_generator
            .fetch_add(1, Ordering::Relaxed)
            .to_be_bytes();
        [a, b, c]
    }
}
