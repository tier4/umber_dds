use crate::dds::tokens::*;
use crate::discovery::{discovery_db::DiscoveryDB, structure::builtin_endpoint::BuiltinEndpoint};
use crate::rtps::reader::{Reader, ReaderIngredients};
use crate::rtps::writer::{Writer, WriterIngredients};
use crate::structure::{
    entity::RTPSEntity,
    entity_id::EntityId,
    guid::*,
    proxy::{ReaderProxy, WriterProxy},
};
use bytes::BytesMut;
use mio_extras::{channel as mio_channel, timer::Timer};
use mio_v06::net::UdpSocket;
use mio_v06::{Events, Poll, PollOpt, Ready, Token};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use crate::message::message_receiver::*;
use crate::message::submessage::element::Locator;
use crate::network::{net_util::*, udp_sender::UdpSender};

const MAX_MESSAGE_SIZE: usize = 64 * 1024; // This is max we can get from UDP.
const MESSAGE_BUFFER_ALLOCATION_CHUNK: usize = 256 * 1024;

pub struct EventLoop {
    domain_id: u16,
    poll: Poll,
    sockets: HashMap<Token, UdpSocket>,
    message_receiver: MessageReceiver,
    add_writer_receiver: mio_channel::Receiver<WriterIngredients>,
    add_reader_receiver: mio_channel::Receiver<ReaderIngredients>,
    set_reader_hb_timer_sender: mio_channel::Sender<(EntityId, GUID)>,
    set_reader_hb_timer_receiver: mio_channel::Receiver<(EntityId, GUID)>,
    writers: HashMap<EntityId, Writer>,
    readers: HashMap<EntityId, Reader>,
    sender: Rc<UdpSender>,
    writer_hb_timer: Timer<EntityId>,
    reader_hb_timers: Vec<Timer<(EntityId, GUID)>>, // (reader EntityId, writer GUID)
    discdb_update_receiver: mio_channel::Receiver<GuidPrefix>,
    discovery_db: DiscoveryDB,
}

impl EventLoop {
    pub fn new(
        domain_id: u16,
        mut sockets: HashMap<Token, UdpSocket>,
        participant_guidprefix: GuidPrefix,
        mut add_writer_receiver: mio_channel::Receiver<WriterIngredients>,
        mut add_reader_receiver: mio_channel::Receiver<ReaderIngredients>,
        discovery_db: DiscoveryDB,
        mut discdb_update_receiver: mio_channel::Receiver<GuidPrefix>,
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
        let mut writer_hb_timer = Timer::default();
        poll.register(
            &mut writer_hb_timer,
            WRITER_HEARTBEAT_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        poll.register(
            &mut discdb_update_receiver,
            DISCOVERY_DB_UPDATE,
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        let (set_reader_hb_timer_sender, mut set_reader_hb_timer_receiver) = mio_channel::channel();
        poll.register(
            &mut set_reader_hb_timer_receiver,
            SET_READER_HEARTBEAT_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        let message_receiver = MessageReceiver::new(participant_guidprefix);
        let sender = Rc::new(UdpSender::new(0).expect("coludn't gen sender"));
        EventLoop {
            domain_id,
            poll,
            sockets,
            message_receiver,
            add_writer_receiver,
            add_reader_receiver,
            set_reader_hb_timer_sender,
            set_reader_hb_timer_receiver,
            writers: HashMap::new(),
            readers: HashMap::new(),
            sender,
            writer_hb_timer,
            reader_hb_timers: Vec::new(),
            discdb_update_receiver,
            discovery_db,
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
                                if writer.guid().entity_id
                                    == EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER
                                {
                                    writer.matched_reader_add(
                                        GUID::UNKNOW,
                                        false,
                                        Vec::new(),
                                        Vec::from([Locator::new_from_ipv4(
                                            spdp_multicast_port(self.domain_id) as u32,
                                            [239, 255, 0, 1],
                                        )]),
                                    );
                                }
                                if writer.is_reliable() {
                                    self.writer_hb_timer
                                        .set_timeout(writer.heartbeat_period(), writer.entity_id());
                                }
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
                                let reader = Reader::new(
                                    reader_ing,
                                    self.sender.clone(),
                                    self.set_reader_hb_timer_sender.clone(),
                                );
                                self.readers.insert(reader.entity_id(), reader);
                            }
                        }
                        DISCOVERY_SEND_TOKEN => {
                            eprintln!("=== @event_loop Discovery cmd received ===");
                            todo!(); // send spdp msg
                        }
                        DISCOVERY_DB_UPDATE => {
                            self.handle_participant_discovery();
                        }
                        WRITER_HEARTBEAT_TIMER => {
                            if let Some(eid) = self.writer_hb_timer.poll() {
                                if let Some(writer) = self.writers.get_mut(&eid) {
                                    writer.send_heart_beat();
                                    self.writer_hb_timer
                                        .set_timeout(writer.heartbeat_period(), writer.entity_id());
                                };
                            }
                        }
                        SET_READER_HEARTBEAT_TIMER => {
                            while let Ok((reader_entity_id, writer_guid)) =
                                self.set_reader_hb_timer_receiver.try_recv()
                            {
                                if let Some(reader) = self.readers.get(&reader_entity_id) {
                                    let mut reader_hb_timer = Timer::default();
                                    reader_hb_timer.set_timeout(
                                        reader.heartbeat_response_delay(),
                                        (reader_entity_id, writer_guid),
                                    );
                                    self.poll
                                        .register(
                                            &mut reader_hb_timer,
                                            WRITER_HEARTBEAT_TIMER,
                                            Ready::readable(),
                                            PollOpt::edge(),
                                        )
                                        .unwrap();
                                    self.reader_hb_timers.push(reader_hb_timer);
                                }
                            }
                        }
                        READER_HEARTBEAT_TIMER => {
                            for rhb_timer in &mut self.reader_hb_timers {
                                if let Some((reid, wguid)) = rhb_timer.poll() {
                                    if let Some(reader) = self.readers.get_mut(&reid) {
                                        reader.handle_hb_response_timeout(wguid);
                                    }
                                }
                            }
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

    fn handle_participant_discovery(&mut self) {
        // configure sedp_builtin_{pub/sub}_writer based on reseived spdp_data
        eprintln!("##################  @discovery  Discovery message received",);

        while let Ok(guid_prefix) = self.discdb_update_receiver.try_recv() {
            if let Some(spdp_data) = self.discovery_db.read(guid_prefix) {
                eprintln!("spdp from {:?} received.", spdp_data.guid);
                if spdp_data.domain_id != self.domain_id {
                    continue;
                } else {
                    let available_builtin_endpoint = spdp_data.available_builtin_endpoint;
                    if available_builtin_endpoint
                        .contains(BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
                    {
                        let guid = GUID::new(
                            spdp_data.guid.guid_prefix,
                            EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                        );
                        if let Some(writer) = self
                            .writers
                            .get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)
                        {
                            writer.matched_reader_add(
                                guid,
                                spdp_data.expects_inline_qos,
                                spdp_data.metarraffic_unicast_locator_list.clone(),
                                spdp_data.metarraffic_multicast_locator_list.clone(),
                            );
                        }
                    }
                    if available_builtin_endpoint
                        .contains(BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
                    {
                        let guid = GUID::new(
                            spdp_data.guid.guid_prefix,
                            EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                        );
                        if let Some(reader) = self
                            .readers
                            .get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR)
                        {
                            reader.matched_writer_add(
                                guid,
                                spdp_data.metarraffic_unicast_locator_list.clone(),
                                spdp_data.metarraffic_multicast_locator_list.clone(),
                                0, // TODO: What value should I set?
                            );
                        }
                    }
                    if available_builtin_endpoint
                        .contains(BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
                    {
                        let guid = GUID::new(
                            spdp_data.guid.guid_prefix,
                            EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                        );
                        if let Some(writer) = self
                            .writers
                            .get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER)
                        {
                            writer.matched_reader_add(
                                guid,
                                spdp_data.expects_inline_qos,
                                spdp_data.metarraffic_unicast_locator_list.clone(),
                                spdp_data.metarraffic_multicast_locator_list.clone(),
                            );
                        }
                    }
                    if available_builtin_endpoint
                        .contains(BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
                    {
                        let guid = GUID::new(
                            spdp_data.guid.guid_prefix,
                            EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                        );
                        if let Some(reader) = self
                            .readers
                            .get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR)
                        {
                            reader.matched_writer_add(
                                guid,
                                spdp_data.metarraffic_unicast_locator_list,
                                spdp_data.metarraffic_multicast_locator_list,
                                0, // TODO: What value should I set?
                            );
                        }
                    }
                }
            }
        }
    }

    fn remove_discoverd_participant(&mut self, participant_guid: GUID) {
        let guid = GUID::new(
            participant_guid.guid_prefix,
            EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
        );
        if let Some(sedp_builtin_pub_writer) = self
            .writers
            .get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)
        {
            sedp_builtin_pub_writer.matched_reader_remove(guid);
        }

        let guid = GUID::new(
            participant_guid.guid_prefix,
            EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        );
        if let Some(sedp_builtin_pub_reader) = self
            .readers
            .get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR)
        {
            sedp_builtin_pub_reader.matched_writer_remove(guid);
        }

        let guid = GUID::new(
            participant_guid.guid_prefix,
            EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        if let Some(sedp_builtin_sub_writer) = self
            .writers
            .get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER)
        {
            sedp_builtin_sub_writer.matched_reader_remove(guid);
        }

        let guid = GUID::new(
            participant_guid.guid_prefix,
            EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        );
        if let Some(sedp_builtin_sub_reader) = self
            .readers
            .get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR)
        {
            sedp_builtin_sub_reader.matched_writer_remove(guid);
        }
    }
}
