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
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::thread;

#[derive(Clone)]
pub struct DomainParticipant {
    inner: Arc<DomainParticipantInner>,
}

impl RTPSEntity for DomainParticipant {
    fn guid(&self) -> GUID {
        self.inner.my_guid
    }
}

impl DomainParticipant {
    pub fn new(domain_id: u16) -> Self {
        Self {
            inner: Arc::new(DomainParticipantInner::new(domain_id)),
        }
    }
    pub fn create_publisher(&self, qos: QosPolicies) -> Publisher {
        self.inner.create_publisher(self.clone(), qos)
    }
    pub fn create_subscriber(&self, qos: QosPolicies) -> Subscriber {
        self.inner.create_subscriber(self.clone(), qos)
    }
    pub fn gen_entity_key(&self) -> [u8; 3] {
        self.inner.gen_entity_key()
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
        let discovery = Discovery {};
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

        let mut participant_id = 0;
        let mut discovery_uni: Option<UdpSocket> = None;
        while discovery_uni.is_none() && participant_id < 120 {
            match new_unicast("0.0.0.0", spdp_unicast_port(domain_id, participant_id)) {
                Ok(s) => discovery_uni = Some(s),
                Err(_e) => participant_id += 1,
            }
        }

        let discovery_uni = match discovery_uni {
            Some(s) => s,
            None => panic!("the max number of participant on same host on same domin is 127."),
        };

        socket_list.insert(DISCOVERY_UNI_TOKEN, discovery_multi);
        socket_list.insert(DISCOVERY_MULTI_TOKEN, discovery_uni);

        let (add_writer_sender, add_writer_receiver) =
            mio_channel::sync_channel::<WriterIngredients>(10);
        let (add_reader_sender, add_reader_receiver) =
            mio_channel::sync_channel::<ReaderIngredients>(10);

        let new_thread = thread::spawn(move || {
            let guid_prefix = GuidPrefix::new();
            let ev_loop = EventLoop::new(
                socket_list,
                guid_prefix,
                add_writer_receiver,
                add_reader_receiver,
            );
            ev_loop.event_loop();
        });

        let my_guid = GUID::new_participant_guid();

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
            EntityId::new_with_entity_kind(&dp, EntityKind::PUBLISHER),
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
            EntityId::new_with_entity_kind(&dp, EntityKind::SUBSCRIBER),
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
