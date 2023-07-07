use crate::network::net_util::*;
use crate::rtps::writer::*;
use crate::structure::entity::RTPSEntity;
use crate::{
    dds::{event_loop::EventLoop, publisher::Publisher, qos::QosPolicies, subscriber::Subscriber},
    network::udp_listinig_socket::*,
    structure::guid::*,
};
use mio::net::UdpSocket;
use mio_channel;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
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
    fn create_publisher(&self, qos: QosPolicies) -> Publisher {
        self.inner.create_publisher(self.clone(), qos)
    }
    fn create_subscriber(&self, qos: QosPolicies) -> Subscriber {
        self.inner.create_subscriber(qos)
    }
}

pub struct DomainParticipantInner {
    domain_id: u16,
    participant_id: u16,
    pub my_guid: GUID,
    add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
    thread: thread::JoinHandle<()>,
}

impl DomainParticipantInner {
    pub fn new(domain_id: u16) -> DomainParticipantInner {
        let mut socket_list: HashMap<mio::Token, UdpSocket> = HashMap::new();
        let spdp_multi_socket = new_multicast(
            "0.0.0.0",
            spdp_multicast_port(domain_id),
            Ipv4Addr::new(239, 255, 0, 1),
        );
        let spdp_uni_socket = new_unicast("0.0.0.0", spdp_unicast_port(domain_id, 0));
        socket_list.insert(DISCOVERY_UNI_TOKEN, spdp_uni_socket);
        socket_list.insert(DISCOVERY_MUTI_TOKEN, spdp_multi_socket);

        let (add_writer_sender, add_writer_receiver) =
            mio_channel::sync_channel::<WriterIngredients>(10);

        let new_thread = thread::spawn(|| {
            let guid_prefix = GuidPrefix::new();
            let ev_loop = EventLoop::new(socket_list, guid_prefix, add_writer_receiver);
            ev_loop.event_loop();
        });

        let my_guid = GUID::new_participant_guid();

        Self {
            domain_id,
            participant_id: 0,
            my_guid,
            add_writer_sender,
            thread: new_thread,
        }
    }

    fn create_publisher(&self, dp: DomainParticipant, qos: QosPolicies) -> Publisher {
        // add_writer用のチャネルを生やして、senderはpubにreceiverは自分
        Publisher::new(qos.clone(), qos.clone(), dp, self.add_writer_sender.clone())
    }
    fn create_subscriber(&self, qos: QosPolicies) -> Subscriber {
        Subscriber::new(qos)
    }
}
