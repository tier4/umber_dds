use mio::*;
use std::collections::HashMap;
use mio::net::UdpSocket;
use mio::{Poll, Events, Interest, Token};
use std::net::SocketAddr;

use crate::network::net_util::*;

pub struct EventLoop {
    poll: Poll,
    sockets: HashMap::<Token, UdpSocket>,
}

impl EventLoop {
    pub fn new(mut sockets: HashMap::<Token, UdpSocket> ) -> EventLoop {
        let poll = Poll::new().unwrap();
        for (token, lister) in &mut sockets {
            poll.registry().register(lister, *token, Interest::READABLE).unwrap();
        }
        EventLoop { poll , sockets }
    }

    pub fn event_loop(mut self) {
        let mut events = Events::with_capacity(1024);
        loop {
            self.poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match event.token() {
                    DISCOVERY_MUTI_TOKEN => {
                        println!("DISCOVERY_MUTI_TOKEN");
                        let udp_sock = self.sockets.get_mut(&event.token()).unwrap();
                        let mut buf = [0; 1024];
                        let (num_of_byte, src_addr) = udp_sock.recv_from(&mut buf).unwrap();
                        for i in 0..10 {
                            print!("{:02X} ", buf[i]);
                        }
                    }
                    DISCOVERY_UNI_TOKEN => {
                        println!("DISCOVERY_UNI_TOKEN");
                        let udp_sock = self.sockets.get_mut(&event.token()).unwrap();
                        let mut buf = [0; 1024];
                        let (num_of_byte, src_addr) = udp_sock.recv_from(&mut buf).unwrap();
                        for i in 0..10 {
                            print!("{:02X} ", buf[i]);
                        }
                    }
                    _ => {println!("undefined event")}
                }
            }
        }
    }

    fn handle_packe() {}
}
