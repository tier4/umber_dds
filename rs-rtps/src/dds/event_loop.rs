use crate::dds::tokens::*;
use crate::discovery::discovery_db::DiscoveryDB;
use crate::rtps::reader::{Reader, ReaderIngredients};
use crate::rtps::writer::{Writer, WriterIngredients};
use crate::structure::{entity::RTPSEntity, entity_id::EntityId, guid::*};
use bytes::BytesMut;
use mio_extras::{channel as mio_channel, timer::Timer};
use mio_v06::net::UdpSocket;
use mio_v06::{Events, Poll, PollOpt, Ready, Token};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use crate::message::message_receiver::*;
use crate::network::{net_util::*, udp_sender::UdpSender};

const MAX_MESSAGE_SIZE: usize = 64 * 1024; // This is max we can get from UDP.
const MESSAGE_BUFFER_ALLOCATION_CHUNK: usize = 256 * 1024;

pub struct EventLoop {
    poll: Poll,
    sockets: HashMap<Token, UdpSocket>,
    message_receiver: MessageReceiver,
    add_writer_receiver: mio_channel::Receiver<WriterIngredients>,
    add_reader_receiver: mio_channel::Receiver<ReaderIngredients>,
    writers: HashMap<EntityId, Writer>,
    readers: HashMap<EntityId, Reader>,
    sender: Rc<UdpSender>,
    spdp_send_timer: Timer<()>,
}

impl EventLoop {
    pub fn new(
        mut sockets: HashMap<Token, UdpSocket>,
        participant_guidprefix: GuidPrefix,
        mut add_writer_receiver: mio_channel::Receiver<WriterIngredients>,
        mut add_reader_receiver: mio_channel::Receiver<ReaderIngredients>,
        discovery_db: DiscoveryDB,
        discdb_update_sender: mio_channel::Sender<GuidPrefix>,
    ) -> EventLoop {
        let poll = Poll::new().unwrap();
        for (token, lister) in &mut sockets {
            poll.register(lister, *token, Ready::readable(), PollOpt::edge())
                .unwrap();
        }
        poll.register(
            &mut add_writer_receiver,
            ADD_WRITER_TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        poll.register(
            &mut add_reader_receiver,
            ADD_READER_TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        let mut spdp_send_timer: Timer<()> = Timer::default();
        // spdp_send_timer.set_timeout(Duration::new(20, 0), ()); // TODO:
        // spdpの送信間隔をtunableにする。
        poll.register(
            &mut spdp_send_timer,
            DISCOVERY_SEND_TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        let message_receiver =
            MessageReceiver::new(participant_guidprefix, discovery_db, discdb_update_sender);
        let sender = Rc::new(UdpSender::new(0).expect("coludn't gen sender"));
        EventLoop {
            poll,
            sockets,
            message_receiver,
            add_writer_receiver,
            add_reader_receiver,
            writers: HashMap::new(),
            readers: HashMap::new(),
            sender,
            spdp_send_timer,
        }
    }

    pub fn event_loop(mut self) {
        let mut events = Events::with_capacity(1024);
        loop {
            self.poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match TokenDec::decode(event.token()) {
                    TokenDec::ReservedToken(token) => match token {
                        DISCOVERY_MULTI_TOKEN | DISCOVERY_UNI_TOKEN => {
                            let udp_sock = self.sockets.get_mut(&event.token()).unwrap();
                            let packets = EventLoop::receiv_packet(udp_sock);
                            self.message_receiver.handle_packet(
                                packets,
                                &mut self.writers,
                                &mut self.readers,
                            );
                        }
                        USERTRAFFIC_MULTI_TOKEN | USERTRAFFIC_UNI_TOKEN => {
                            let udp_sock = self.sockets.get_mut(&event.token()).unwrap();
                            let packets = EventLoop::receiv_packet(udp_sock);
                            self.message_receiver.handle_packet(
                                packets,
                                &mut self.writers,
                                &mut self.readers,
                            );
                        }
                        ADD_WRITER_TOKEN => {
                            while let Ok(writer_ing) = self.add_writer_receiver.try_recv() {
                                eprintln!("in ev_loop: entity_id: {:?}", writer_ing.guid);
                                let mut writer = Writer::new(writer_ing, self.sender.clone());
                                let token = writer.entity_token();
                                self.poll
                                    .register(
                                        &mut writer.writer_command_receiver,
                                        token,
                                        Ready::readable(),
                                        PollOpt::edge(),
                                    )
                                    .unwrap();
                                self.writers.insert(writer.entity_id(), writer);
                            }
                        }
                        ADD_READER_TOKEN => {
                            while let Ok(reader_ing) = self.add_reader_receiver.try_recv() {
                                eprintln!("in event_loop received add reader");
                                let reader = Reader::new(reader_ing);
                                self.readers.insert(reader.entity_id(), reader);
                            }
                        }
                        DISCOVERY_SEND_TOKEN => {
                            eprintln!("=== @event_loop Discovery cmd received ===");
                            todo!(); // send spdp msg
                        }
                        _ => eprintln!("undefined Token or unimplemented event"),
                    },
                    TokenDec::Entity(eid) => {
                        if eid.is_writer() {
                            eprintln!("~~~~~~~~~~~~~Writer entity: {:?}", eid);
                            let writer = match self.writers.get_mut(&eid) {
                                Some(w) => w,
                                None => panic!("Unregisterd writer."),
                            };
                            writer.handle_writer_cmd();
                        } else if eid.is_reader() {
                            unimplemented!("Reader entity: {:?}", eid);
                        } else {
                            panic!("receive message from unknown entity: {:?}", eid);
                        }
                    }
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
