use crate::dds::tokens::*;
use crate::rtps::writer::{Writer, WriterCmd, WriterIngredients};
use crate::structure::{entity::RTPSEntity, entity_id::EntityId, guid::*};
use bytes::BytesMut;
use mio::net::UdpSocket;
use mio::{Events, Interest, Poll, Token};
use mio_channel;
use std::collections::HashMap;

use crate::message::message_receiver::*;
use crate::network::net_util::*;

const MAX_MESSAGE_SIZE: usize = 64 * 1024; // This is max we can get from UDP.
const MESSAGE_BUFFER_ALLOCATION_CHUNK: usize = 256 * 1024;

pub struct EventLoop {
    poll: Poll,
    sockets: HashMap<Token, UdpSocket>,
    message_receiver: MessageReceiver,
    add_writer_receiver: mio_channel::Receiver<WriterIngredients>,
    writers: HashMap<EntityId, Writer>,
}

impl EventLoop {
    pub fn new(
        mut sockets: HashMap<Token, UdpSocket>,
        participant_guidprefix: GuidPrefix,
        mut add_writer_receiver: mio_channel::Receiver<WriterIngredients>,
    ) -> EventLoop {
        let poll = Poll::new().unwrap();
        for (token, lister) in &mut sockets {
            poll.registry()
                .register(lister, *token, Interest::READABLE)
                .unwrap();
        }
        poll.registry()
            .register(
                &mut add_writer_receiver,
                ADD_WRITER_TOKEN,
                Interest::READABLE,
            )
            .unwrap();
        let message_receiver = MessageReceiver::new(participant_guidprefix);
        EventLoop {
            poll,
            sockets,
            message_receiver,
            add_writer_receiver,
            writers: HashMap::new(),
        }
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
                        self.message_receiver.handle_packet(packets);
                    }
                    ADD_WRITER_TOKEN => {
                        let writer_ing = self.add_writer_receiver.try_recv().unwrap();
                        // TODO: writer_cmd_receiver とwriterの紐づけ&pollに登録
                        let mut writer = Writer::new(writer_ing);
                        let token = writer.entity_token();
                        self.poll
                            .registry()
                            .register(
                                &mut writer.writer_command_receiver,
                                token,
                                Interest::READABLE,
                            )
                            .unwrap();
                        self.writers.insert(writer.entity_id(), writer);
                    }
                    _ => println!("undefined event"),
                }
            }
        }
    }

    fn receiv_packet(udp_sock: &UdpSocket) -> Vec<UdpMessage> {
        let mut packets: Vec<UdpMessage> = Vec::with_capacity(4);
        loop {
            let mut buf = BytesMut::with_capacity(MESSAGE_BUFFER_ALLOCATION_CHUNK);
            unsafe {
                buf.set_len(MAX_MESSAGE_SIZE);
            }
            let (num_of_byte, addr) = match udp_sock.recv_from(&mut buf) {
                Ok((n, addr)) => (n, addr),
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        // pass
                    } else {
                        panic!();
                    }
                    return packets;
                }
            };
            let mut packet = buf.split_to(buf.len());
            packet.truncate(num_of_byte);
            packets.push(UdpMessage {
                message: packet,
                addr,
            });
        }
    }
}
