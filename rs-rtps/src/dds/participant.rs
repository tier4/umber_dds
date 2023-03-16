use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::thread;
use mio::net::UdpSocket;
use crate::network::net_util::*;
use crate::{
    network::udp_listinig_socket::*,
    dds::event_loop::EventLoop,
    structure::guid::*};

pub struct DomainParticipant {
    domain_id: u16,
    participant_id: u16,
    thread: thread::JoinHandle<()>,
}

impl DomainParticipant {
    pub fn new(domain_id: u16) -> DomainParticipant {
        let mut socket_list: HashMap<mio::Token, UdpSocket> = HashMap::new();
        let spdp_multi_socket = new_multicast("0.0.0.0", spdp_multicast_port(domain_id), Ipv4Addr::new(239, 255, 0, 1));
        let spdp_uni_socket = new_unicast("0.0.0.0", spdp_unicast_port(domain_id, 0));
        socket_list.insert(DISCOVERY_UNI_TOKEN, spdp_uni_socket);
        socket_list.insert(DISCOVERY_MUTI_TOKEN, spdp_multi_socket);

        
        let new_thread = thread::spawn(|| {
            let guid_prefix = GuidPrefix::new();
            let ev_loop = EventLoop::new(socket_list, guid_prefix);
            ev_loop.event_loop();
        });

        DomainParticipant { domain_id, participant_id: 0, thread: new_thread }
    }
}
