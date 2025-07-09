use crate::dds::qos::{
    policy::{Durability, History, HistoryQosKind, LivelinessQosKind, Reliability},
    DataReaderQosBuilder, DataWriterQosBuilder,
};
use crate::dds::tokens::*;
use crate::discovery::{
    discovery_db::{DiscoveryDB, EndpointState},
    structure::builtin_endpoint::BuiltinEndpoint,
    structure::data::{DiscoveredReaderData, DiscoveredWriterData},
    DiscoveryDBUpdateNotifier,
};
use crate::rtps::reader::{Reader, ReaderIngredients};
use crate::rtps::writer::{Writer, WriterIngredients};
use crate::structure::{Duration, EntityId, GuidPrefix, RTPSEntity, GUID};
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use bytes::BytesMut;
use core::time::Duration as CoreDuration;
use log::{debug, error, info, trace};
use mio_extras::{
    channel as mio_channel,
    timer::{Timeout, Timer},
};
use mio_v06::net::UdpSocket;
use mio_v06::{Events, Poll, PollOpt, Ready, Token};

use crate::message::message_receiver::*;
use crate::message::submessage::element::{Locator, Timestamp};
use crate::network::{net_util::*, udp_sender::UdpSender};

const MAX_MESSAGE_SIZE: usize = 64 * 1024; // This is max we can get from UDP.
const MESSAGE_BUFFER_ALLOCATION_CHUNK: usize = 256 * 1024;
const ASSERT_LIVELINESS_PERIOD: u64 = 10;

pub struct EventLoop {
    domain_id: u16,
    guid_prefix: GuidPrefix,
    poll: Poll,
    sockets: BTreeMap<Token, UdpSocket>,
    message_receiver: MessageReceiver,
    // receive writer ingredients from publisher
    create_writer_receiver: mio_channel::Receiver<WriterIngredients>,
    // receive writer ingredients from subscriber
    create_reader_receiver: mio_channel::Receiver<ReaderIngredients>,
    // notify new writer to discovery module
    notify_new_writer_sender: mio_channel::Sender<(EntityId, DiscoveredWriterData)>,
    // notify new reader to discovery module
    notify_new_reader_sender: mio_channel::Sender<(EntityId, DiscoveredReaderData)>,
    // to distribute to reader
    set_reader_hb_timer_sender: mio_channel::Sender<(EntityId, GUID)>,
    // receive heartbeat_response_delay timer from reader
    set_reader_hb_timer_receiver: mio_channel::Receiver<(EntityId, GUID)>,
    // to distribute to writer
    set_writer_nack_timer_sender: mio_channel::Sender<(EntityId, GUID)>,
    // receive nack_response_delay timer from writer
    set_writer_nack_timer_receiver: mio_channel::Receiver<(EntityId, GUID)>,
    writers: BTreeMap<EntityId, Writer>,
    readers: BTreeMap<EntityId, Reader>,
    udp_sender: Rc<UdpSender>,
    writer_hb_timer: Timer<EntityId>,
    reader_hb_timers: Vec<Timer<(EntityId, GUID)>>, // (reader EntityId, writer GUID)
    writer_nack_timers: Vec<Timer<(EntityId, GUID)>>, // (writer EntityId, reader GUID)
    wlp_timer_receiver: mio_channel::Receiver<EntityId>,
    wlp_timers: BTreeMap<EntityId, (Timer<EntityId>, Timeout)>, //  reader EntityId, reader EntityId
    assert_liveliness_timer: Timer<()>,
    check_liveliness_timer: Timer<Vec<GUID>>,
    check_liveliness_timer_to: Option<(CoreDuration, Timeout)>,
    // receive discovery_db update notification from Discovery
    discdb_update_receiver: mio_channel::Receiver<DiscoveryDBUpdateNotifier>,
    discovery_db: DiscoveryDB,
}

impl EventLoop {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain_id: u16,
        guid_prefix: GuidPrefix,
        mut sockets: BTreeMap<Token, UdpSocket>,
        udp_sender: UdpSender,
        participant_guidprefix: GuidPrefix,
        create_writer_receiver: mio_channel::Receiver<WriterIngredients>,
        create_reader_receiver: mio_channel::Receiver<ReaderIngredients>,
        notify_new_writer_sender: mio_channel::Sender<(EntityId, DiscoveredWriterData)>,
        notify_new_reader_sender: mio_channel::Sender<(EntityId, DiscoveredReaderData)>,
        discovery_db: DiscoveryDB,
        discdb_update_receiver: mio_channel::Receiver<DiscoveryDBUpdateNotifier>,
    ) -> EventLoop {
        let poll = Poll::new().unwrap();
        for (token, lister) in &mut sockets {
            poll.register(lister, *token, Ready::readable(), PollOpt::edge())
                .expect("coludn't register lister to poll");
        }
        poll.register(
            &create_writer_receiver,
            ADD_WRITER_TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("coludn't register create_writer_receiver to poll");
        poll.register(
            &create_reader_receiver,
            ADD_READER_TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("coludn't register create_reader_receiver to poll");
        let writer_hb_timer = Timer::default();
        poll.register(
            &writer_hb_timer,
            WRITER_HEARTBEAT_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("coludn't register writer_hb_timer to poll");
        let mut assert_liveliness_timer = Timer::default();
        assert_liveliness_timer.set_timeout(CoreDuration::from_secs(ASSERT_LIVELINESS_PERIOD), ());
        trace!(
            "set Writer assert_liveliness timer({})",
            ASSERT_LIVELINESS_PERIOD
        );
        poll.register(
            &assert_liveliness_timer,
            ASSERT_AUTOMATIC_LIVELINESS_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("coludn't register assert_liveliness_timer to poll");
        let check_liveliness_timer = Timer::default();
        poll.register(
            &check_liveliness_timer,
            CHECK_MANUAL_LIVELINESS_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("coludn't register check_liveliness_timer to poll");
        poll.register(
            &discdb_update_receiver,
            DISCOVERY_DB_UPDATE,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("coludn't register discdb_update_receiver to poll");
        let (set_reader_hb_timer_sender, set_reader_hb_timer_receiver) = mio_channel::channel();
        let (set_writer_nack_timer_sender, set_writer_nack_timer_receiver) = mio_channel::channel();
        let (wlp_timer_sender, wlp_timer_receiver) = mio_channel::channel();
        poll.register(
            &set_reader_hb_timer_receiver,
            SET_READER_HEARTBEAT_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("coludn't register set_reader_hb_timer_receiver to poll");
        poll.register(
            &set_writer_nack_timer_receiver,
            SET_WRITER_NACK_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("coludn't register set_writer_nack_timer_receiver to poll");
        poll.register(
            &wlp_timer_receiver,
            SET_WLP_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("coludn't register wlp_timer_receiver to poll");
        let message_receiver = MessageReceiver::new(participant_guidprefix, wlp_timer_sender);
        EventLoop {
            domain_id,
            guid_prefix,
            poll,
            sockets,
            message_receiver,
            create_writer_receiver,
            create_reader_receiver,
            notify_new_writer_sender,
            notify_new_reader_sender,
            set_reader_hb_timer_sender,
            set_reader_hb_timer_receiver,
            set_writer_nack_timer_sender,
            set_writer_nack_timer_receiver,
            writers: BTreeMap::new(),
            readers: BTreeMap::new(),
            udp_sender: Rc::new(udp_sender),
            writer_hb_timer,
            reader_hb_timers: Vec::new(),
            writer_nack_timers: Vec::new(),
            wlp_timer_receiver,
            wlp_timers: BTreeMap::new(),
            assert_liveliness_timer,
            check_liveliness_timer,
            check_liveliness_timer_to: None,
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
                            if let Some(udp_sock) = self.sockets.get_mut(&event.token()) {
                                let packets = EventLoop::receiv_packet(udp_sock);
                                self.message_receiver.handle_packet(
                                    packets,
                                    &mut self.writers,
                                    &mut self.readers,
                                    &mut self.discovery_db,
                                );
                            }
                        }
                        USERTRAFFIC_MULTI_TOKEN | USERTRAFFIC_UNI_TOKEN => {
                            if let Some(udp_sock) = self.sockets.get_mut(&event.token()) {
                                let packets = EventLoop::receiv_packet(udp_sock);
                                self.message_receiver.handle_packet(
                                    packets,
                                    &mut self.writers,
                                    &mut self.readers,
                                    &mut self.discovery_db,
                                );
                            }
                        }
                        ADD_WRITER_TOKEN => {
                            while let Ok(writer_ing) = self.create_writer_receiver.try_recv() {
                                let mut writer = Writer::new(
                                    writer_ing,
                                    self.udp_sender.clone(),
                                    self.set_writer_nack_timer_sender.clone(),
                                );
                                if writer.entity_id()
                                    == EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER
                                {
                                    // this is for sending discovery message
                                    // readerEntityId of SPDP message from Rust DDS:
                                    // ENTITYID_BUILT_IN_SDP_PARTICIPANT_READER
                                    // readerEntityId of SPDP message from Cyclone DDS:
                                    // ENTITYID_UNKNOW
                                    // stpr 2.3 sepc, 9.6.1.4, Default multicast address
                                    let qos = DataReaderQosBuilder::new()
                                        .reliability(Reliability::default_besteffort())
                                        .build();
                                    writer.matched_reader_add(
                                        GUID::UNKNOW, // this is same to CycloneDDS
                                        false,
                                        Vec::new(),
                                        Vec::from([Locator::new_from_ipv4(
                                            spdp_multicast_port(self.domain_id) as u32,
                                            [239, 255, 0, 1],
                                        )]),
                                        qos,
                                    );
                                }
                                if writer.is_reliable() {
                                    trace!(
                                        "set Writer Heartbeat timer({:?}) of Writer with {}",
                                        writer.heartbeat_period(),
                                        writer.entity_id(),
                                    );
                                    self.writer_hb_timer
                                        .set_timeout(writer.heartbeat_period(), writer.entity_id());
                                }
                                let qos = writer.get_qos();
                                match qos.liveliness().kind {
                                    LivelinessQosKind::Automatic => (),
                                    LivelinessQosKind::ManualByTopic
                                    | LivelinessQosKind::ManualByParticipant => {
                                        let ld = qos.liveliness().lease_duration;
                                        if ld != Duration::INFINITE {
                                            if let Some((d, to)) =
                                                self.check_liveliness_timer_to.as_ref()
                                            {
                                                if let Some(mut w) =
                                                    self.check_liveliness_timer.cancel_timeout(to)
                                                {
                                                    w.push(writer.guid());
                                                    let duration = std::cmp::min(
                                                        *d,
                                                        ld.half().to_core_duration(),
                                                    );
                                                    let to = self
                                                        .check_liveliness_timer
                                                        .set_timeout(duration, w);
                                                    self.check_liveliness_timer_to =
                                                        Some((duration, to));
                                                    trace!(
                                                        "set Writer check_liveliness timer({:?})",
                                                        duration
                                                    );
                                                }
                                            } else {
                                                let duration = ld.half().to_core_duration();
                                                let to = self
                                                    .check_liveliness_timer
                                                    .set_timeout(duration, vec![writer.guid()]);
                                                self.check_liveliness_timer_to =
                                                    Some((duration, to));
                                                trace!(
                                                    "set Writer check_liveliness timer({:?})",
                                                    duration
                                                );
                                            }
                                        }
                                    }
                                }
                                let token = writer.entity_token();
                                self.poll
                                    .register(
                                        &writer.writer_command_receiver,
                                        token,
                                        Ready::readable(),
                                        PollOpt::edge(),
                                    )
                                    .expect(
                                        "coludn't register writer.writer_command_receiver to poll",
                                    );
                                if writer.entity_id()
                                    != EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER
                                    && writer.entity_id()
                                        != EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
                                    && writer.entity_id()
                                        != EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
                                    && writer.entity_id()
                                        != EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER
                                {
                                    self.notify_new_writer_sender
                                        .send((writer.entity_id(), writer.sedp_data()))
                                        .expect("coludn't send channel 'notify_new_writer_sender'");
                                }
                                self.writers.insert(writer.entity_id(), writer);
                            }
                        }
                        ADD_READER_TOKEN => {
                            while let Ok(reader_ing) = self.create_reader_receiver.try_recv() {
                                let reader = Reader::new(
                                    reader_ing,
                                    self.udp_sender.clone(),
                                    self.set_reader_hb_timer_sender.clone(),
                                );
                                if reader.entity_id() != EntityId::SPDP_BUILTIN_PARTICIPANT_DETECTOR
                                    && reader.entity_id()
                                        != EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR
                                    && reader.entity_id()
                                        != EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
                                    && reader.entity_id()
                                        != EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER
                                {
                                    self.notify_new_reader_sender
                                        .send((reader.entity_id(), reader.sedp_data()))
                                        .expect("coludn't send channle 'notify_new_reader_sender'");
                                }
                                self.readers.insert(reader.entity_id(), reader);
                            }
                        }
                        DISCOVERY_SEND_TOKEN => {
                            info!("=== @event_loop Discovery cmd received ===");
                            todo!(); // send spdp msg
                        }
                        DISCOVERY_DB_UPDATE => {
                            self.handle_participant_discovery();
                        }
                        WRITER_HEARTBEAT_TIMER => {
                            while let Some(eid) = self.writer_hb_timer.poll() {
                                trace!("fired Writer Heartbeat timer({})", eid);
                                if let Some(writer) = self.writers.get_mut(&eid) {
                                    writer.send_heart_beat(false);
                                    self.writer_hb_timer
                                        .set_timeout(writer.heartbeat_period(), writer.entity_id());
                                    trace!(
                                        "set Writer Heartbeat timer({:?}) of Writer with {}",
                                        writer.heartbeat_period(),
                                        writer.entity_id(),
                                    );
                                } else {
                                    error!("not found Writer which fired Heartbeat timer\n\tWriter: {}", eid);
                                }
                            }
                        }
                        SET_READER_HEARTBEAT_TIMER => {
                            while let Ok((reader_entity_id, writer_guid)) =
                                self.set_reader_hb_timer_receiver.try_recv()
                            {
                                if let Some(reader) = self.readers.get(&reader_entity_id) {
                                    let mut reader_hb_timer = Timer::default();
                                    trace!(
                                        "set Heartbeat response delay timer({:?}) of Reader\n\tReader: {}\n\tWriter: {}",
                                        reader.heartbeat_response_delay(),
                                        reader_entity_id,
                                        writer_guid
                                    );
                                    reader_hb_timer.set_timeout(
                                        reader.heartbeat_response_delay(),
                                        (reader_entity_id, writer_guid),
                                    );
                                    self.poll
                                        .register(
                                            &reader_hb_timer,
                                            WRITER_HEARTBEAT_TIMER,
                                            Ready::readable(),
                                            PollOpt::edge(),
                                        )
                                        .expect("coludn't register reader_hb_timer to poll");
                                    self.reader_hb_timers.push(reader_hb_timer);
                                } else {
                                    error!("not found Reader which attempt to set heartbeat timer\n\tReader: {}", reader_entity_id);
                                }
                            }
                        }
                        SET_WRITER_NACK_TIMER => {
                            while let Ok((writer_entity_id, reader_guid)) =
                                self.set_writer_nack_timer_receiver.try_recv()
                            {
                                if let Some(writer) = self.writers.get(&writer_entity_id) {
                                    let mut writedr_an_timer = Timer::default();
                                    trace!(
                                        "set Writer AckNack timer({:?})\n\tWriter: {}\n\tReader: {}",
                                        writer.nack_response_delay(),
                                        writer_entity_id,
                                        reader_guid
                                    );
                                    writedr_an_timer.set_timeout(
                                        writer.nack_response_delay(),
                                        (writer_entity_id, reader_guid),
                                    );
                                    self.poll
                                        .register(
                                            &writedr_an_timer,
                                            WRITER_NACK_TIMER,
                                            Ready::readable(),
                                            PollOpt::edge(),
                                        )
                                        .expect("coludn't register writedr_an_timer to poll");
                                    self.writer_nack_timers.push(writedr_an_timer);
                                } else {
                                    error!("not found Writer which attempt to set nack response timer\n\tWriter: {}", writer_entity_id);
                                }
                            }
                        }
                        WRITER_LIVELINESS_CHECK_TIMER => {
                            for (wlp_timer, to) in self.wlp_timers.values_mut() {
                                if let Some(eid) = wlp_timer.poll() {
                                    trace!(
                                        "fired Reader liveliness check timer\n\tReader: {}",
                                        eid
                                    );
                                    if let Some(reader) = self.readers.get_mut(&eid) {
                                        reader.check_liveliness(&mut self.discovery_db);
                                        trace!(
                                            "checked liveliness of Reader\n\tReader: {}",
                                            reader.entity_id()
                                        );
                                        let time = reader.get_min_remote_writer_lease_duration();
                                        *to = wlp_timer.set_timeout(time, reader.entity_id());
                                        trace!(
                                            "set Reader liveliness check timer({:?})\n\tReader: {}",
                                            time,
                                            reader.entity_id()
                                        );
                                    } else {
                                        error!("not found Reader which fired check liveliness timer\n\tReader: {}", eid);
                                    }
                                }
                            }
                        }
                        ASSERT_AUTOMATIC_LIVELINESS_TIMER => {
                            self.assert_liveliness_timer.poll();
                            trace!("fired Writer assert_liveliness timer");
                            let now = Timestamp::now().unwrap_or(Timestamp::TIME_INVALID);
                            for writer in self.writers.values_mut() {
                                let guid = writer.guid();
                                if let EndpointState::Live(ts) =
                                    self.discovery_db.read_local_writer(guid)
                                {
                                    let duration = now - ts;
                                    let liveliness = writer.get_qos().liveliness();
                                    if liveliness.kind != LivelinessQosKind::Automatic {
                                        continue;
                                    }
                                    if liveliness.lease_duration == Duration::INFINITE {
                                        continue;
                                    }
                                    if duration > liveliness.lease_duration.half() {
                                        writer.assert_liveliness();
                                    }
                                } else {
                                    debug!("not found local Writer which attempt to assert liveliness\n\tWriter: {}", guid);
                                }
                            }
                            self.assert_liveliness_timer
                                .set_timeout(CoreDuration::from_secs(ASSERT_LIVELINESS_PERIOD), ());
                            trace!(
                                "set Writer assert_liveliness timer({})",
                                ASSERT_LIVELINESS_PERIOD
                            );
                        }
                        CHECK_MANUAL_LIVELINESS_TIMER => {
                            trace!("fired Writer check_liveliness timer");
                            while let Some(wgs) = self.check_liveliness_timer.poll() {
                                for wg in &wgs {
                                    if let Some(w) = self.writers.get_mut(&wg.entity_id) {
                                        trace!(
                                            "checked liveliness of local Writer\n\tWriter: {}",
                                            wg.entity_id
                                        );
                                        w.check_liveliness();
                                    }
                                }
                                let duration = self.check_liveliness_timer_to.unwrap().0;
                                let to = self.check_liveliness_timer.set_timeout(duration, wgs);
                                self.check_liveliness_timer_to = Some((duration, to));
                                trace!(
                                    "set Writer check_liveliness timer({})",
                                    ASSERT_LIVELINESS_PERIOD
                                );
                            }
                        }
                        READER_HEARTBEAT_TIMER => {
                            for rhb_timer in &mut self.reader_hb_timers {
                                if let Some((reid, wguid)) = rhb_timer.poll() {
                                    trace!("fired Reader Heartbeat timer\n\tReader: {}", reid);
                                    if let Some(reader) = self.readers.get_mut(&reid) {
                                        reader.handle_hb_response_timeout(wguid);
                                    } else {
                                        error!("not found Reader which fired Heartbeat timer\n\tReader: {}", reid);
                                    }
                                }
                            }
                        }
                        WRITER_NACK_TIMER => {
                            for wan_timer in &mut self.writer_nack_timers {
                                if let Some((weid, rguid)) = wan_timer.poll() {
                                    trace!("fired Writer AckNack timer\n\tWriter: {}", weid);
                                    if let Some(writer) = self.writers.get_mut(&weid) {
                                        writer.handle_nack_response_timeout(rguid);
                                    } else {
                                        error!("not found Writer which fired Heartbeat timer\n\tWriter: {}", weid);
                                    }
                                }
                            }
                        }
                        SET_WLP_TIMER => {
                            while let Ok(reader_eid) = self.wlp_timer_receiver.try_recv() {
                                if let Some(reader) = self.readers.get_mut(&reader_eid) {
                                    let min_ld = reader.get_min_remote_writer_lease_duration();
                                    let mut wlp_timer = Timer::default();
                                    let timeout = wlp_timer.set_timeout(min_ld, reader.entity_id());
                                    self.poll
                                        .register(
                                            &wlp_timer,
                                            WRITER_LIVELINESS_CHECK_TIMER,
                                            Ready::readable(),
                                            PollOpt::edge(),
                                        )
                                        .expect("coludn't register reader_hb_timer to poll");
                                    if let Some((timer, to)) =
                                        self.wlp_timers.get_mut(&reader.entity_id())
                                    {
                                        timer.cancel_timeout(to);
                                        reader.check_liveliness(&mut self.discovery_db);
                                    }
                                    self.wlp_timers
                                        .insert(reader.entity_id(), (wlp_timer, timeout));
                                } else {
                                    error!("not found Reader which attempt to set WriterLivelinessTimer\n\tReader: {}", reader_eid);
                                }
                            }
                        }
                        Token(n) => info!("@event_loop: Token(0x{:02X}) is not implemented", n),
                    },
                    TokenDec::Entity(eid) => {
                        if eid.is_writer() {
                            if let Some(writer) = self.writers.get_mut(&eid) {
                                self.discovery_db.write_local_writer(
                                    GUID::new(self.guid_prefix, eid),
                                    Timestamp::now().expect("failed get Timestamp"),
                                    writer.get_qos().liveliness().kind,
                                );
                                writer.handle_writer_cmd();
                            } else {
                                error!(
                                    "EventLoop's poll received event with Token of unregisterd Writer {}",
                                    eid
                                );
                            }
                        } else if eid.is_reader() {
                            unreachable!(
                                "EventLoop's poll received event with TokenDec::Entity(Reader entityid)"
                            );
                        } else {
                            unreachable!(
                                "EventLoop's poll received event with TokenDec::Entity(UNKNOW entityid)"
                            );
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

        while let Ok(discdb_update) = self.discdb_update_receiver.try_recv() {
            match discdb_update {
                DiscoveryDBUpdateNotifier::AddParticipant(guid_prefix) => {
                    trace!("handle_participant_discovery: {}", guid_prefix);
                    if let Some(spdp_data) = self.discovery_db.read_participant_data(guid_prefix) {
                        if spdp_data.domain_id != self.domain_id {
                            continue;
                        } else {
                            let available_builtin_endpoint = spdp_data.available_builtin_endpoint;
                            if available_builtin_endpoint.contains(
                                BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR,
                            ) {
                                let guid = GUID::new(
                                    spdp_data.guid.guid_prefix,
                                    EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                                );
                                if let Some(writer) = self
                                    .writers
                                    .get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER)
                                {
                                    trace!(
                                        "sedp_writer.matched_reader_add(remote_sedp_pub_reader)"
                                    );
                                    let qos = DataReaderQosBuilder::new()
                                        .reliability(Reliability::default_besteffort())
                                        .build();
                                    writer.matched_reader_add(
                                        guid,
                                        spdp_data.expects_inline_qos,
                                        spdp_data.metarraffic_unicast_locator_list.clone(),
                                        spdp_data.metarraffic_multicast_locator_list.clone(),
                                        qos,
                                    );
                                } else {
                                    error!("not found self::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER");
                                }
                            }
                            if available_builtin_endpoint.contains(
                                BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER,
                            ) {
                                let guid = GUID::new(
                                    spdp_data.guid.guid_prefix,
                                    EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                                );
                                if let Some(reader) = self
                                    .readers
                                    .get_mut(&EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR)
                                {
                                    trace!(
                                        "sedp_reader.matched_writer_add(remote_sedp_pub_writer)"
                                    );
                                    let qos = DataWriterQosBuilder::new()
                                        .reliability(Reliability::default_reliable())
                                        .build();
                                    reader.matched_writer_add(
                                        guid,
                                        spdp_data.metarraffic_unicast_locator_list.clone(),
                                        spdp_data.metarraffic_multicast_locator_list.clone(),
                                        0, // TODO: What value should I set?
                                        qos,
                                    );
                                } else {
                                    error!("not found self::SEDP_BUILTIN_PUBLICATIONS_DETECTOR");
                                }
                            }
                            if available_builtin_endpoint.contains(
                                BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR,
                            ) {
                                let guid = GUID::new(
                                    spdp_data.guid.guid_prefix,
                                    EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                                );
                                if let Some(writer) = self
                                    .writers
                                    .get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER)
                                {
                                    trace!(
                                        "sedp_writer.matched_reader_add(remote_sedp_sub_reader)"
                                    );
                                    let qos = DataReaderQosBuilder::new()
                                        .reliability(Reliability::default_reliable())
                                        .build();
                                    writer.matched_reader_add(
                                        guid,
                                        spdp_data.expects_inline_qos,
                                        spdp_data.metarraffic_unicast_locator_list.clone(),
                                        spdp_data.metarraffic_multicast_locator_list.clone(),
                                        qos,
                                    );
                                } else {
                                    error!("not found self::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER");
                                }
                            }
                            if available_builtin_endpoint.contains(
                                BuiltinEndpoint::DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER,
                            ) {
                                let guid = GUID::new(
                                    spdp_data.guid.guid_prefix,
                                    EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                                );
                                if let Some(reader) = self
                                    .readers
                                    .get_mut(&EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR)
                                {
                                    trace!(
                                        "sedp_reader.matched_writer_add(remote_sedp_sub_writer)"
                                    );
                                    let qos = DataWriterQosBuilder::new()
                                        .reliability(Reliability::default_reliable())
                                        .build();
                                    reader.matched_writer_add(
                                        guid,
                                        spdp_data.metarraffic_unicast_locator_list.clone(),
                                        spdp_data.metarraffic_multicast_locator_list.clone(),
                                        0, // TODO: What value should I set?
                                        qos,
                                    );
                                } else {
                                    error!("not found self::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR");
                                }
                            }
                            if available_builtin_endpoint.contains(
                                BuiltinEndpoint::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER,
                            ) {
                                let guid = GUID::new(
                                    spdp_data.guid.guid_prefix,
                                    EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
                                );
                                if let Some(writer) = self
                                    .writers
                                    .get_mut(&EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER)
                                {
                                    trace!(
                                        "p2p_msg_writer.matched_reader_add(remote_p2p_msg_reader)"
                                    );
                                    // rtps 2.3 sepc, 8.4.13.3 BuiltinParticipantMessageWriter and BuiltinParticipantMessageReader QoS
                                    // If the ParticipantProxy::builtinEndpointQos is included in the SPDPdiscoveredParticipantData, then the
                                    // BuiltinParticipantMessageWriter shall treat the BuiltinParticipantMessageReader as indicated by the flags If the
                                    // ParticipantProxy::builtinEndpointQos is not included then the BuiltinParticipantMessageWriter shall treat the
                                    // BuiltinParticipantMessageReader as if it is configured with RELIABLE_RELIABILITY_QOS.
                                    let qos = DataReaderQosBuilder::new()
                                        .reliability(Reliability::default_reliable()) // TODO: set best_effort if the flag indicated
                                        .durability(Durability::TransientLocal)
                                        .history(History {
                                            kind: HistoryQosKind::KeepLast,
                                            depth: 1,
                                        })
                                        .build();
                                    writer.matched_reader_add(
                                        guid,
                                        spdp_data.expects_inline_qos,
                                        spdp_data.metarraffic_unicast_locator_list.clone(),
                                        spdp_data.metarraffic_multicast_locator_list.clone(),
                                        qos,
                                    );
                                } else {
                                    error!(
                                        "not found self::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER"
                                    );
                                }
                            }
                            if available_builtin_endpoint.contains(
                                BuiltinEndpoint::BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER,
                            ) {
                                let guid = GUID::new(
                                    spdp_data.guid.guid_prefix,
                                    EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
                                );
                                if let Some(reader) = self
                                    .readers
                                    .get_mut(&EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER)
                                {
                                    trace!(
                                        "p2p_msg_reader.matched_writer_add(remote_p2p_msg_writer)"
                                    );
                                    let qos = DataWriterQosBuilder::new()
                                        .reliability(Reliability::default_reliable())
                                        .durability(Durability::TransientLocal)
                                        .history(History {
                                            kind: HistoryQosKind::KeepLast,
                                            depth: 1,
                                        })
                                        .build();
                                    reader.matched_writer_add(
                                        guid,
                                        spdp_data.metarraffic_unicast_locator_list,
                                        spdp_data.metarraffic_multicast_locator_list,
                                        0, // TODO: What value should I set?
                                        qos,
                                    );
                                } else {
                                    error!(
                                        "not found self::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER"
                                    );
                                }
                            }
                        }
                    }
                }
                DiscoveryDBUpdateNotifier::DeleteParticipant(guid_prefix) => {
                    info!("Participant lost\n\tParticipant: {}", guid_prefix);
                    self.remove_discoverd_participant(guid_prefix);
                }
            }
        }
    }

    fn remove_discoverd_participant(&mut self, participant_guidp: GuidPrefix) {
        for (_eid, r) in self.readers.iter_mut() {
            r.delete_writer_proxy(participant_guidp);
        }
        for (_eid, w) in self.writers.iter_mut() {
            w.delete_reader_proxy(participant_guidp);
        }
    }
}
