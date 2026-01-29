use crate::dds::qos::{
    policy::{LivelinessQosKind, Reliability},
    DataReaderQosBuilder,
};
use crate::dds::tokens::*;
use crate::discovery::{
    discovery_db::{DiscoveryDB, EndpointState},
    structure::data::{DiscoveredReaderData, DiscoveredWriterData},
    BuiltinEndpointsIngredients, DiscoveryDBUpdateNotifier,
};
use crate::rtps::reader::{Reader, ReaderIngredients, ReaderTimer};
use crate::rtps::writer::{Writer, WriterIngredients, WriterTimer};
use crate::structure::{Duration, EntityId, GuidPrefix, RTPSEntity, GUID};
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use bytes::BytesMut;
use core::time::Duration as CoreDuration;
use log::{error, info, trace};
use mio_extras::{
    channel as mio_channel,
    timer::{Timeout, Timer},
};
use mio_v06::net::UdpSocket;
use mio_v06::{Events, Poll, PollOpt, Ready, Token};

use crate::message::message_receiver::*;
use crate::message::submessage::element::{Locator, SerializedPayload, Timestamp};
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
    set_reader_timer_sender: mio_channel::Sender<ReaderTimer>,
    // receive reader timer from reader
    set_reader_timer_receiver: mio_channel::Receiver<ReaderTimer>,
    // to distribute to writer
    set_writer_timer_sender: mio_channel::Sender<WriterTimer>,
    // receive writer timer from writer
    set_writer_timer_receiver: mio_channel::Receiver<WriterTimer>,
    writers: BTreeMap<EntityId, Writer>,
    readers: BTreeMap<EntityId, Reader>,
    udp_sender: Rc<UdpSender>,
    writer_hb_timer: Timer<EntityId>,
    reader_hb_timer: Timer<(EntityId, GUID)>, // (reader EntityId, writer GUID)
    reader_deadline_timer: Timer<((EntityId, GUID), CoreDuration)>, // (reader EntityId, writer GUID)
    reader_deadline_timeout: BTreeMap<(EntityId, GUID), Timeout>, // (reader EntityId, writer GUID)
    writer_nack_timer: Timer<(EntityId, GUID)>,                   // (writer EntityId, reader GUID)
    writer_deadline_timer: Timer<(EntityId, CoreDuration)>,
    writer_deadline_timeout: BTreeMap<EntityId, Timeout>,
    wlp_timer_receiver: mio_channel::Receiver<EntityId>,
    wlp_timer: Timer<EntityId>,                //  reader EntityId
    wlp_timeouts: BTreeMap<EntityId, Timeout>, //  reader EntityId
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
        spdp_data: SerializedPayload,
        builtin_endpoints_ingredients: BuiltinEndpointsIngredients,
    ) -> EventLoop {
        let poll = Poll::new().unwrap();
        for (token, lister) in &mut sockets {
            poll.register(lister, *token, Ready::readable(), PollOpt::edge())
                .expect("failed to register UdpSocket lister with poll");
        }
        poll.register(
            &create_writer_receiver,
            ADD_WRITER_TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register receiver 'create_writer_receiver' with poll");
        poll.register(
            &create_reader_receiver,
            ADD_READER_TOKEN,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register receiver 'create_reader_receiver' with poll");
        let writer_hb_timer = Timer::default();
        poll.register(
            &writer_hb_timer,
            WRITER_HEARTBEAT_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register timer 'writer_hb_timer' with poll");
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
        .expect("failed to register timer 'assert_liveliness_timer' with poll");
        let check_liveliness_timer = Timer::default();
        poll.register(
            &check_liveliness_timer,
            CHECK_MANUAL_LIVELINESS_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register timer 'check_liveliness_timer' with poll");
        poll.register(
            &discdb_update_receiver,
            DISCOVERY_DB_UPDATE,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register receiver 'discdb_update_receiver' with poll");
        let (set_reader_timer_sender, set_reader_timer_receiver) = mio_channel::channel();
        let (set_writer_timer_sender, set_writer_timer_receiver) = mio_channel::channel();
        let (wlp_timer_sender, wlp_timer_receiver) = mio_channel::channel();
        poll.register(
            &set_reader_timer_receiver,
            SET_READER_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register receiver 'set_reader_hb_timer_receiver' with poll");
        poll.register(
            &set_writer_timer_receiver,
            SET_WRITER_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register receiver 'set_writer_nack_timer_receiver' with poll");
        poll.register(
            &wlp_timer_receiver,
            SET_WLP_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register receiver 'wlp_timer_receiver' with poll");
        let reader_hb_timer = Timer::default();
        poll.register(
            &reader_hb_timer,
            READER_HEARTBEAT_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register timer 'reader_hb_timer' with poll");
        let reader_deadline_timer = Timer::default();
        poll.register(
            &reader_deadline_timer,
            READER_DEADLINE_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register timer 'reader_deadline_timer' with poll");
        let writer_nack_timer = Timer::default();
        poll.register(
            &writer_nack_timer,
            WRITER_NACK_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register timer 'writer_nack_timer' with poll");
        let writer_deadline_timer = Timer::default();
        poll.register(
            &writer_deadline_timer,
            WRITER_DEADLINE_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register timer 'writer_deadline_timer' with poll");
        let wlp_timer = Timer::default();
        poll.register(
            &wlp_timer,
            WRITER_LIVELINESS_CHECK_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("failed to register timer 'wlp_timer' with poll");
        let message_receiver = MessageReceiver::new(
            participant_guidprefix,
            domain_id,
            discovery_db.clone(),
            wlp_timer_sender,
            spdp_data,
        );
        let mut ev_loop = EventLoop {
            domain_id,
            guid_prefix,
            poll,
            sockets,
            message_receiver,
            create_writer_receiver,
            create_reader_receiver,
            notify_new_writer_sender,
            notify_new_reader_sender,
            set_reader_timer_sender,
            set_reader_timer_receiver,
            set_writer_timer_sender,
            set_writer_timer_receiver,
            writers: BTreeMap::new(),
            readers: BTreeMap::new(),
            udp_sender: Rc::new(udp_sender),
            writer_hb_timer,
            reader_hb_timer,
            reader_deadline_timer,
            reader_deadline_timeout: BTreeMap::new(),
            writer_nack_timer,
            writer_deadline_timer,
            writer_deadline_timeout: BTreeMap::new(),
            wlp_timer_receiver,
            wlp_timer,
            wlp_timeouts: BTreeMap::new(),
            assert_liveliness_timer,
            check_liveliness_timer,
            check_liveliness_timer_to: None,
            discdb_update_receiver,
            discovery_db,
        };
        ev_loop.register_builtin_endpoints(builtin_endpoints_ingredients);
        ev_loop
    }

    fn register_builtin_endpoints(
        &mut self,
        builtin_endpoints_ingredients: BuiltinEndpointsIngredients,
    ) {
        self.register_writer(builtin_endpoints_ingredients.spdp_builtin_participant_writer_ing);
        // self.register_reader(builtin_endpoints_ingredients.spdp_builtin_participant_reader_ing);
        self.register_writer(builtin_endpoints_ingredients.sedp_builtin_pub_writer_ing);
        self.register_reader(builtin_endpoints_ingredients.sedp_builtin_pub_reader_ing);
        self.register_writer(builtin_endpoints_ingredients.sedp_builtin_sub_writer_ing);
        self.register_reader(builtin_endpoints_ingredients.sedp_builtin_sub_reader_ing);
        self.register_writer(builtin_endpoints_ingredients.p2p_builtin_participant_msg_writer_ing);
        self.register_reader(builtin_endpoints_ingredients.p2p_builtin_participant_msg_reader_ing);
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
                                );
                            }
                        }
                        ADD_WRITER_TOKEN => {
                            while let Ok(writer_ing) = self.create_writer_receiver.try_recv() {
                                self.register_writer(writer_ing);
                            }
                        }
                        ADD_READER_TOKEN => {
                            while let Ok(reader_ing) = self.create_reader_receiver.try_recv() {
                                self.register_reader(reader_ing);
                            }
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
                                        "set Writer Heartbeat timer({:?})\n\tWriter: {}",
                                        writer.heartbeat_period(),
                                        writer.entity_id(),
                                    );
                                } else {
                                    error!("not found Writer from EventLoop.writers which fired Heartbeat timer\n\tWriter: {}", eid);
                                }
                            }
                        }
                        SET_READER_TIMER => {
                            while let Ok(reader_timer) = self.set_reader_timer_receiver.try_recv() {
                                match reader_timer {
                                    ReaderTimer::Heartbeat(reader_entity_id, writer_guid) => {
                                        if let Some(reader) = self.readers.get(&reader_entity_id) {
                                            trace!(
                                                "set Heartbeat response delay timer({:?}) of Reader\n\tReader: {}\n\tWriter: {}",
                                                reader.heartbeat_response_delay(),
                                                reader_entity_id,
                                                writer_guid
                                            );
                                            self.reader_hb_timer.set_timeout(
                                                reader.heartbeat_response_delay(),
                                                (reader_entity_id, writer_guid),
                                            );
                                        } else {
                                            error!("not found Reader from EventLoop.readers which attempt to set heartbeat timer\n\tReader: {}", reader_entity_id);
                                        }
                                    }
                                    ReaderTimer::Deadline(
                                        reader_entity_id,
                                        writer_guid,
                                        duration,
                                    ) => {
                                        if let Some(to) = self
                                            .reader_deadline_timeout
                                            .get(&(reader_entity_id, writer_guid))
                                        {
                                            trace!(
                                                "cancel Writer Deadline timer({:?})\n\tReader: {}\n\tWriter: {}",
                                                duration,
                                                reader_entity_id,
                                                writer_guid,
                                            );
                                            self.reader_deadline_timer.cancel_timeout(to);
                                        }
                                        trace!(
                                            "set Reader Deadline timer({:?})\n\tReader: {}\n\tWriter: {}",
                                            duration,
                                            reader_entity_id,
                                            writer_guid,
                                        );
                                        let to = self.reader_deadline_timer.set_timeout(
                                            duration,
                                            ((reader_entity_id, writer_guid), duration),
                                        );
                                        self.reader_deadline_timeout
                                            .insert((reader_entity_id, writer_guid), to);
                                    }
                                }
                            }
                        }
                        READER_DEADLINE_TIMER => {
                            while let Some(((reid, wguid), duration)) =
                                self.reader_deadline_timer.poll()
                            {
                                trace!(
                                    "fired Reader Deadline timer\n\tReader: {}\n\tWriter: {}",
                                    reid,
                                    wguid
                                );
                                if let Some(reader) = self.readers.get(&reid) {
                                    reader.notify_reqested_deadline_missed();
                                    trace!(
                                        "set Reader Deadline timer({:?})\n\tReader: {}",
                                        duration,
                                        reid,
                                    );
                                    let to = self
                                        .reader_deadline_timer
                                        .set_timeout(duration, ((reid, wguid), duration));
                                    self.reader_deadline_timeout.insert((reid, wguid), to);
                                } else {
                                    unreachable!();
                                }
                            }
                        }
                        SET_WRITER_TIMER => {
                            while let Ok(writer_timer) = self.set_writer_timer_receiver.try_recv() {
                                match writer_timer {
                                    WriterTimer::Nack(writer_entity_id, reader_guid) => {
                                        if let Some(writer) = self.writers.get(&writer_entity_id) {
                                            trace!(
                                                "set Writer AckNack timer({:?})\n\tWriter: {}\n\tReader: {}",
                                                writer.nack_response_delay(),
                                                writer_entity_id,
                                                reader_guid
                                            );
                                            self.writer_nack_timer.set_timeout(
                                                writer.nack_response_delay(),
                                                (writer_entity_id, reader_guid),
                                            );
                                        } else {
                                            error!("not found Writer from EventLoop.writers which attempt to set nack response timer\n\tWriter: {}", writer_entity_id);
                                        }
                                    }
                                    WriterTimer::Deadline(writer_entity_id, duration) => {
                                        if let Some(to) =
                                            self.writer_deadline_timeout.get(&writer_entity_id)
                                        {
                                            trace!(
                                                "cancel Writer Deadline timer({:?})\n\tWriter: {}",
                                                duration,
                                                writer_entity_id,
                                            );
                                            self.writer_deadline_timer.cancel_timeout(to);
                                        }
                                        trace!(
                                            "set Writer Deadline timer({:?})\n\tWriter: {}",
                                            duration,
                                            writer_entity_id,
                                        );
                                        let to = self
                                            .writer_deadline_timer
                                            .set_timeout(duration, (writer_entity_id, duration));
                                        self.writer_deadline_timeout.insert(writer_entity_id, to);
                                    }
                                }
                            }
                        }
                        WRITER_DEADLINE_TIMER => {
                            while let Some((eid, duration)) = self.writer_deadline_timer.poll() {
                                trace!("fired Writer Deadline timer\n\tWriter: {}", eid);
                                if let Some(writer) = self.writers.get(&eid) {
                                    writer.notify_offered_deadline_missed();
                                    trace!(
                                        "set Writer Deadline timer({:?})\n\tWriter: {}",
                                        duration,
                                        eid,
                                    );
                                    let to = self
                                        .writer_deadline_timer
                                        .set_timeout(duration, (eid, duration));
                                    self.writer_deadline_timeout.insert(eid, to);
                                } else {
                                    unreachable!();
                                }
                            }
                        }
                        WRITER_LIVELINESS_CHECK_TIMER => {
                            while let Some(eid) = self.wlp_timer.poll() {
                                trace!("fired Reader liveliness check timer\n\tReader: {}", eid);
                                if let Some(reader) = self.readers.get_mut(&eid) {
                                    reader.check_liveliness(&mut self.discovery_db);
                                    trace!(
                                        "checked liveliness of Reader\n\tReader: {}",
                                        reader.entity_id()
                                    );
                                    let time = reader.get_min_remote_writer_lease_duration();
                                    let to = self.wlp_timer.set_timeout(time, reader.entity_id());
                                    self.wlp_timeouts.insert(reader.entity_id(), to);
                                    trace!(
                                        "set Reader liveliness check timer({:?})\n\tReader: {}",
                                        time,
                                        reader.entity_id()
                                    );
                                } else {
                                    error!("not found Reader from EventLoop.readers which fired check liveliness timer\n\tReader: {}", eid);
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
                                        trace!("assert_liveliness continue because not kind Automatic\n\tWriter: {}", guid);
                                        continue;
                                    }
                                    if liveliness.lease_duration == Duration::INFINITE {
                                        trace!("assert_liveliness continue because duration INFINITE\n\tWriter: {}", guid);
                                        continue;
                                    }
                                    if duration > liveliness.lease_duration.half() {
                                        writer.assert_liveliness();
                                        trace!("assert_liveliness()\n\tWriter: {}", guid);
                                        self.discovery_db.write_local_writer(
                                            writer.guid(),
                                            Timestamp::now()
                                                .expect("failed to get Timestamp::now()"),
                                            writer.get_qos().liveliness().kind,
                                        );
                                    }
                                } else {
                                    error!("failed to assert liveliness of writer: writer not found in discovery_db or its EndpointState is not alive\n\tWriter: {}", guid);
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
                            while let Some((reid, wguid)) = self.reader_hb_timer.poll() {
                                trace!("fired Reader Heartbeat timer\n\tReader: {}", reid);
                                if let Some(reader) = self.readers.get_mut(&reid) {
                                    reader.handle_hb_response_timeout(wguid);
                                } else {
                                    error!("not found Reader which fired Heartbeat timer\n\tReader: {}", reid);
                                }
                            }
                        }
                        WRITER_NACK_TIMER => {
                            while let Some((weid, rguid)) = self.writer_nack_timer.poll() {
                                trace!("fired Writer AckNack timer\n\tWriter: {}", weid);
                                if let Some(writer) = self.writers.get_mut(&weid) {
                                    writer.handle_nack_response_timeout(rguid);
                                } else {
                                    error!("not found Writer which fired Heartbeat timer\n\tWriter: {}", weid);
                                }
                            }
                        }
                        SET_WLP_TIMER => {
                            while let Ok(reader_eid) = self.wlp_timer_receiver.try_recv() {
                                if let Some(reader) = self.readers.get_mut(&reader_eid) {
                                    let min_ld = reader.get_min_remote_writer_lease_duration();
                                    if let Some(to) = self.wlp_timeouts.get_mut(&reader.entity_id())
                                    {
                                        self.wlp_timer.cancel_timeout(to);
                                        reader.check_liveliness(&mut self.discovery_db);
                                    }
                                    let timeout =
                                        self.wlp_timer.set_timeout(min_ld, reader.entity_id());
                                    self.wlp_timeouts.insert(reader.entity_id(), timeout);
                                } else {
                                    error!("not found Reader which attempt to set WriterLivelinessTimer\n\tReader: {}", reader_eid);
                                }
                            }
                        }
                        Token(n) => error!("@event_loop: Token(0x{:02X}) is not implemented", n),
                    },
                    TokenDec::Entity(eid) => {
                        if eid.is_writer() {
                            if let Some(writer) = self.writers.get_mut(&eid) {
                                self.discovery_db.write_local_writer(
                                    GUID::new(self.guid_prefix, eid),
                                    Timestamp::now().expect("failed to get Timestamp::now()"),
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

    fn register_writer(&mut self, writer_ing: WriterIngredients) {
        let mut writer = Writer::new(
            writer_ing,
            self.udp_sender.clone(),
            self.set_writer_timer_sender.clone(),
        );
        if writer.entity_id() == EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER {
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
                "set Writer Heartbeat timer({:?})\n\tWriter: {}",
                writer.heartbeat_period(),
                writer.entity_id(),
            );
            self.writer_hb_timer
                .set_timeout(writer.heartbeat_period(), writer.entity_id());
        }
        let qos = writer.get_qos();
        match qos.liveliness().kind {
            LivelinessQosKind::Automatic => (),
            LivelinessQosKind::ManualByTopic | LivelinessQosKind::ManualByParticipant => {
                let ld = qos.liveliness().lease_duration;
                if ld != Duration::INFINITE {
                    if let Some((d, to)) = self.check_liveliness_timer_to.as_ref() {
                        if let Some(mut w) = self.check_liveliness_timer.cancel_timeout(to) {
                            w.push(writer.guid());
                            let duration = std::cmp::min(*d, ld.half().into());
                            let to = self.check_liveliness_timer.set_timeout(duration, w);
                            self.check_liveliness_timer_to = Some((duration, to));
                            trace!(
                                "set Writer check_liveliness timer({:?})\n\tWriter: {}",
                                duration,
                                writer.entity_id()
                            );
                        }
                    } else {
                        let duration = ld.half().into();
                        let to = self
                            .check_liveliness_timer
                            .set_timeout(duration, vec![writer.guid()]);
                        self.check_liveliness_timer_to = Some((duration, to));
                        trace!(
                            "set Writer check_liveliness timer({:?})\n\tWriter: {}",
                            duration,
                            writer.entity_id()
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
            .expect("failed to register receiver 'writer.writer_command_receiver' with poll");
        if writer.entity_id() != EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER
            && writer.entity_id() != EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER
            && writer.entity_id() != EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER
            && writer.entity_id() != EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER
        {
            self.notify_new_writer_sender
                .send((writer.entity_id(), writer.sedp_data()))
                .expect("failed to send data via channel 'notify_new_writer_sender'");
        }
        self.discovery_db.write_local_writer(
            writer.guid(),
            Timestamp::now().expect("failed to get Timestamp::now()"),
            writer.get_qos().liveliness().kind,
        );
        trace!(
            "new Writer added to writers\n\tWriter: {}",
            writer.entity_id(),
        );
        self.writers.insert(writer.entity_id(), writer);
    }
    fn register_reader(&mut self, reader_ing: ReaderIngredients) {
        let reader = Reader::new(
            reader_ing,
            self.udp_sender.clone(),
            self.set_reader_timer_sender.clone(),
        );
        if reader.entity_id() != EntityId::SPDP_BUILTIN_PARTICIPANT_DETECTOR
            && reader.entity_id() != EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR
            && reader.entity_id() != EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR
            && reader.entity_id() != EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER
        {
            self.notify_new_reader_sender
                .send((reader.entity_id(), reader.sedp_data()))
                .expect("failed to send data via channle 'notify_new_reader_sender'");
        }
        trace!(
            "new Reader added to writers\n\tWriter: {}",
            reader.entity_id(),
        );
        self.readers.insert(reader.entity_id(), reader);
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
                        panic!("{}", e);
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
                DiscoveryDBUpdateNotifier::DeleteParticipant(guid_prefix) => {
                    info!(
                        "remove liveliness lost Participant\n\tParticipant: {}",
                        guid_prefix
                    );
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
