use std::collections::HashMap;
use std::net;
use std::thread;
use mio::net::UdpSocket;
use mio::{Poll, Events};

use crate::{network::{udp_listinig_socket::*, net_util::*}, dds::event_loop::EventLoop};

pub struct DomainParticipant {
    domain_id: u16,
    participant_id: u16,
    thread: thread::JoinHandle<()>,
}

impl DomainParticipant {
    pub fn new(domain_id: u16) -> DomainParticipant {
        let mut socket_list: HashMap<mio::Token, UdpSocket> = HashMap::new();
        let spdp_multi_socket = new_multicast("0.0.0.0", spdp_multicast_port(domain_id));
        let spdp_uni_socket = new_unicast("0.0.0.0", spdp_unicast_port(domain_id, 0));
        socket_list.insert(mio::Token(0), spdp_uni_socket);
        socket_list.insert(mio::Token(1), spdp_multi_socket);

        
        let new_thread = thread::spawn(|| {
            let ev_loop = EventLoop::new(socket_list);
            ev_loop.event_loop();
        });

        DomainParticipant { domain_id, participant_id: 0, thread: new_thread }
    }
}
