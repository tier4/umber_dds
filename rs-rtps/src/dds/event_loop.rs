use mio::*;
use std::collections::HashMap;
use mio::net::UdpSocket;
use mio::{Poll, Events, Interest, Token};
use std::net::SocketAddr;
use bytes::{Bytes, BytesMut};
use crate::rtps::message;

use speedy::Readable;

use crate::network::net_util::*;

const MAX_MESSAGE_SIZE: usize = 64 * 1024; // This is max we can get from UDP.
const MESSAGE_BUFFER_ALLOCATION_CHUNK: usize = 256 * 1024;

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
                    DISCOVERY_MUTI_TOKEN | DISCOVERY_UNI_TOKEN => {
                        let udp_sock = self.sockets.get_mut(&event.token()).unwrap();
                        let packets = EventLoop::receiv_packet(udp_sock);
                        EventLoop::handle_packet(packets);
                    },
                    _ => println!("undefined event"),
                }
            }
        }
    }

    fn receiv_packet(udp_sock: &UdpSocket) -> Vec<BytesMut> {
        let mut packets: Vec<BytesMut> = Vec::with_capacity(4);
        loop {
            let mut buf = BytesMut::with_capacity(MESSAGE_BUFFER_ALLOCATION_CHUNK);
            unsafe {buf.set_len(MAX_MESSAGE_SIZE);}
            let num_of_byte = match udp_sock.recv(&mut buf) {
                Ok(n) => {
                    n
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        // pass
                    } else {
                        panic!();
                    }
                    return packets;
                },
            };
            packets.push(buf);
        }
    }

    fn handle_packet(packets: Vec<BytesMut>) {
        for packet in packets {
            // Is DDSPING
            if packet.len() < 20 {
                if packet.len() >= 16 && packet[0..4] == b"RTPS"[..] && packet[9..16] == b"DDSPING"[..] {
                    println!("Received DDSPING");
                    return;
                }
            }
            // TODO: Process RTPS header
            let packet_buf =packet.freeze();
            let rtps_header_buf = packet_buf.slice(..20);
            let rtps_header = match message::Header::read_from_buffer(&rtps_header_buf) {
                Ok(h) => h,
                Err(e) => panic!("{:?}", e),
            };
            // let rtps_contens = packet[20..];

            // loop {
                // process Submessage
            // }
        }
    }
}
