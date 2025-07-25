use crate::dds::{
    qos::{
        policy::*, DataReaderQos, DataReaderQosBuilder, DataWriterQos, DataWriterQosBuilder,
        PublisherQos, SubscriberQos, TopicQos, TopicQosBuilder,
    },
    tokens::*,
    DataReader, DataReaderStatusChanged, DataWriter, DomainParticipant, Publisher, Subscriber,
};
use crate::discovery::discovery_db::DiscoveryDB;
use crate::discovery::structure::builtin_endpoint::BuiltinEndpoint;
use crate::discovery::structure::data::{
    DiscoveredReaderData, DiscoveredWriterData, ParticipantMessageData, SDPBuiltinData,
    SPDPdiscoveredParticipantData,
};
use crate::message::{
    message_header::ProtocolVersion,
    submessage::element::{Locator, Timestamp},
};
use crate::network::net_util::{
    spdp_multicast_port, spdp_unicast_port, usertraffic_multicast_port, usertraffic_unicast_port,
};
use crate::structure::{Duration, EntityId, GuidPrefix, RTPSEntity, TopicKind, VendorId};
use alloc::collections::BTreeMap;
use core::time::Duration as CoreDuration;
use enumflags2::make_bitflags;
use log::{debug, error, info, trace};
use mio_extras::{channel as mio_channel, timer::Timer};
use mio_v06::{Events, Poll, PollOpt, Ready, Token};

pub mod discovery_db;
pub mod structure;

// SPDPbuiltinParticipantWriter
// RTPS Best-Effort StatelessWriter
// HistoryCacheには
// a single data-object of type SPDPdiscoveredParticipantDataを含む
// data-objectを、事前に設定されたlocatorのリストにParticipantの存在を知らせるためにnetworkに送信する。
// このとき、HistoryCacheに存在するすべてのchangesをすべてのlocatorに送信するStatelessWriter::unsent_changes_resetを定期的に呼び出すことで達成される。
// 事前に設定されたlocatorのリストにはSPDP_WELL_KNOWN_UNICAST_PORT,
// SPDP_WELL_KNOWN_MULTICAST_PORTのポートを使わなければならない。
//
// SPDPbuiltinParticipantReader
// RTPS Reader
// remote ParticipantからSPDPdiscoveredParticipantData announcementを受信する。
//
// SEDPbuiltin{Publication/Sunscription}{Writer/Reader}
// rtps 2.3 spec 8.5.4.2によると、reliableでStatefullな{Writer/Reader}

pub enum DiscoveryDBUpdateNotifier {
    AddParticipant(GuidPrefix),
    DeleteParticipant(GuidPrefix),
}

pub enum ParticipantMessageCmd {
    SendData(ParticipantMessageData),
}

#[allow(dead_code)]
pub struct Discovery {
    dp: DomainParticipant,
    discovery_db: DiscoveryDB,
    discdb_update_sender: mio_channel::Sender<DiscoveryDBUpdateNotifier>,
    poll: Poll,
    publisher: Publisher,
    subscriber: Subscriber,
    spdp_builtin_participant_writer: DataWriter<SPDPdiscoveredParticipantData>,
    spdp_builtin_participant_reader: DataReader<SDPBuiltinData>,
    sedp_builtin_pub_writer: DataWriter<DiscoveredWriterData>,
    sedp_builtin_pub_reader: DataReader<SDPBuiltinData>,
    sedp_builtin_sub_writer: DataWriter<DiscoveredReaderData>,
    sedp_builtin_sub_reader: DataReader<SDPBuiltinData>,
    p2p_builtin_participant_msg_writer: DataWriter<ParticipantMessageData>,
    p2p_builtin_participant_msg_reader: DataReader<ParticipantMessageData>,
    spdp_send_timer: Timer<()>,
    participant_liveliness_timer: Timer<()>,
    local_writers_data: BTreeMap<EntityId, DiscoveredWriterData>,
    notify_new_writer_receiver: mio_channel::Receiver<(EntityId, DiscoveredWriterData)>,
    local_readers_data: BTreeMap<EntityId, DiscoveredReaderData>,
    notify_new_reader_receiver: mio_channel::Receiver<(EntityId, DiscoveredReaderData)>,
    participant_msg_cmd_reveiver: mio_channel::Receiver<ParticipantMessageCmd>,
}

impl Discovery {
    pub fn new(
        dp: DomainParticipant,
        discovery_db: DiscoveryDB,
        discdb_update_sender: mio_channel::Sender<DiscoveryDBUpdateNotifier>,
        notify_new_writer_receiver: mio_channel::Receiver<(EntityId, DiscoveredWriterData)>,
        notify_new_reader_receiver: mio_channel::Receiver<(EntityId, DiscoveredReaderData)>,
        participant_msg_cmd_reveiver: mio_channel::Receiver<ParticipantMessageCmd>,
    ) -> Self {
        let poll = Poll::new().unwrap();
        let publisher = dp.create_publisher(PublisherQos::Default);
        let subscriber = dp.create_subscriber(SubscriberQos::Default);

        // For SPDP
        let spdp_topic = dp.create_builtin_topic(
            "DCPSParticipant".to_string(),
            "SPDPDiscoveredParticipantData".to_string(),
            TopicKind::WithKey,
            TopicQos::Default,
        );
        let spdp_writer_qos = DataWriterQos::Policies(Box::new(
            DataWriterQosBuilder::new()
                .reliability(Reliability::default_besteffort())
                .build(),
        ));
        let spdp_reader_qos = DataReaderQos::Policies(Box::new(
            DataReaderQosBuilder::new()
                .reliability(Reliability::default_besteffort())
                .build(),
        ));
        let spdp_writer_entity_id = EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;
        let spdp_reader_entity_id = EntityId::SPDP_BUILTIN_PARTICIPANT_DETECTOR;
        let spdp_builtin_participant_writer = publisher.create_datawriter_with_entityid(
            spdp_writer_qos,
            spdp_topic.clone(),
            spdp_writer_entity_id,
        );
        let spdp_builtin_participant_reader = subscriber.create_datareader_with_entityid(
            spdp_reader_qos,
            spdp_topic.clone(),
            spdp_reader_entity_id,
        );
        poll.register(
            &spdp_builtin_participant_reader,
            SPDP_PARTICIPANT_DETECTOR,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("couldn't register spdp_builtin_participant_reader to poll");

        // For SEDP
        let sedp_writer_qos = DataWriterQos::Policies(Box::new(
            DataWriterQosBuilder::new()
                .reliability(Reliability::default_reliable())
                .build(),
        ));
        let sedp_reader_qos = DataReaderQos::Policies(Box::new(
            DataReaderQosBuilder::new()
                .reliability(Reliability::default_reliable())
                .build(),
        ));
        let sedp_topic_qos = TopicQos::Policies(Box::new(
            TopicQosBuilder::new()
                .reliability(Reliability::default_reliable())
                .build(),
        ));
        let sedp_publication_topic = dp.create_builtin_topic(
            "DCPSPublication".to_string(),
            "PublicationBuiltinTopicData".to_string(),
            TopicKind::WithKey,
            sedp_topic_qos.clone(),
        );
        let sedp_pub_writer_entity_id = EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER;
        let sedp_pub_reader_entity_id = EntityId::SEDP_BUILTIN_PUBLICATIONS_DETECTOR;
        let sedp_builtin_pub_writer: DataWriter<DiscoveredWriterData> = publisher
            .create_datawriter_with_entityid(
                sedp_writer_qos.clone(),
                sedp_publication_topic.clone(),
                sedp_pub_writer_entity_id,
            );
        let sedp_builtin_pub_reader: DataReader<SDPBuiltinData> = subscriber
            .create_datareader_with_entityid(
                sedp_reader_qos.clone(),
                sedp_publication_topic.clone(),
                sedp_pub_reader_entity_id,
            );
        let sedp_subscription_topic = dp.create_builtin_topic(
            "DCPSSucscription".to_string(),
            "SubscriptionBuiltinTopicData".to_string(),
            TopicKind::WithKey,
            sedp_topic_qos,
        );
        let sedp_sub_writer_entity_id = EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER;
        let sedp_sub_reader_entity_id = EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR;
        let sedp_builtin_sub_writer: DataWriter<DiscoveredReaderData> = publisher
            .create_datawriter_with_entityid(
                sedp_writer_qos,
                sedp_subscription_topic.clone(),
                sedp_sub_writer_entity_id,
            );
        let sedp_builtin_sub_reader: DataReader<SDPBuiltinData> = subscriber
            .create_datareader_with_entityid(
                sedp_reader_qos,
                sedp_subscription_topic.clone(),
                sedp_sub_reader_entity_id,
            );

        // For Writer Liveliness Protocol
        let p2p_builtin_participant_topic_qos = TopicQosBuilder::new()
            .reliability(Reliability::default_reliable())
            .durability(Durability::TransientLocal)
            .history(History {
                kind: HistoryQosKind::KeepLast,
                depth: 1,
            })
            .build();
        // rtps 2.3 sepc, 8.4.13.3 BuiltinParticipantMessageWriter and BuiltinParticipantMessageReader QoS
        // > For interoperability, both the BuiltinParticipantMessageWriter and BuiltinParticipantMessageReader shall use the following QoS values:
        // > + durability.kind = TRANSIENT_LOCAL_DURABILITY
        //
        // But Durability QoS of BuiltinParticipantMessage{Writer/Reader} of Cyclone DDS and RustDDS is Volatile.
        // So, in this implementation use Volatile.
        let p2p_builtin_participant_topic = dp.create_builtin_topic(
            "DCPSParticipantMessage".to_string(),
            "ParticipantMessageData".to_string(),
            TopicKind::WithKey,
            TopicQos::Policies(Box::new(p2p_builtin_participant_topic_qos)),
        );
        let p2p_builtin_participant_msg_writer: DataWriter<ParticipantMessageData> = publisher
            .create_datawriter_with_entityid(
                DataWriterQos::Policies(Box::new(publisher.get_default_datawriter_qos())),
                p2p_builtin_participant_topic.clone(),
                EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER,
            );
        let p2p_builtin_participant_msg_reader: DataReader<ParticipantMessageData> = subscriber
            .create_datareader_with_entityid(
                DataReaderQos::Policies(Box::new(subscriber.get_default_datareader_qos())),
                p2p_builtin_participant_topic,
                EntityId::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER,
            );
        poll.register(
            &p2p_builtin_participant_msg_reader,
            PARTICIPANT_MESSAGE_READER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("couldn't register p2p_builtin_participant_msg_reader to poll");

        let mut spdp_send_timer: Timer<()> = Timer::default();
        spdp_send_timer.set_timeout(CoreDuration::new(3, 0), ());
        poll.register(
            &spdp_send_timer,
            SPDP_SEND_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("couldn't register spdp_send_timer to poll");
        let mut participant_liveliness_timer: Timer<()> = Timer::default();
        participant_liveliness_timer.set_timeout(CoreDuration::new(5, 0), ());
        poll.register(
            &participant_liveliness_timer,
            PARTICIPANT_LIVELINESS_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("couldn't register participant_liveliness_timer to poll");
        poll.register(
            &notify_new_writer_receiver,
            DISC_WRITER_ADD,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("couldn't register notify_new_writer_receiver to poll");
        poll.register(
            &notify_new_reader_receiver,
            DISC_READER_ADD,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("couldn't register notify_new_reader_receiver to poll");
        poll.register(
            &participant_msg_cmd_reveiver,
            PARTICIPANT_MESSAGE_CMD_RECEIVER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("couldn't register notify_new_reader_receiver to poll");
        Self {
            dp,
            discovery_db,
            discdb_update_sender,
            poll,
            publisher,
            subscriber,
            spdp_builtin_participant_writer,
            spdp_builtin_participant_reader,
            sedp_builtin_pub_writer,
            sedp_builtin_pub_reader,
            sedp_builtin_sub_writer,
            sedp_builtin_sub_reader,
            p2p_builtin_participant_msg_writer,
            p2p_builtin_participant_msg_reader,
            spdp_send_timer,
            participant_liveliness_timer,
            local_writers_data: BTreeMap::new(),
            notify_new_writer_receiver,
            local_readers_data: BTreeMap::new(),
            notify_new_reader_receiver,
            participant_msg_cmd_reveiver,
        }
    }

    pub fn discovery_loop(&mut self) {
        let mut events = Events::with_capacity(1024);
        let domain_id = self.dp.domain_id();
        let participant_id = self.dp.participant_id();
        let nics = self.dp.get_network_interfaces();
        // This data is for test
        let data = SPDPdiscoveredParticipantData::new(
            self.dp.domain_id(),
            String::from("todo"),
            ProtocolVersion::PROTOCOLVERSION,
            self.dp.guid(),
            VendorId::THIS_IMPLEMENTATION,
            false,
            make_bitflags!(BuiltinEndpoint::{DISC_BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER}),
            Locator::new_list_from_multi_ipv4(
                spdp_unicast_port(domain_id, participant_id) as u32,
                nics.clone(),
            ),
            vec![Locator::new_from_ipv4(
                spdp_multicast_port(domain_id) as u32,
                [239, 255, 0, 1],
            )],
            Locator::new_list_from_multi_ipv4(
                usertraffic_unicast_port(domain_id, participant_id) as u32,
                nics,
            ),
            vec![Locator::new_from_ipv4(
                usertraffic_multicast_port(domain_id) as u32,
                [239, 255, 0, 1],
            )],
            Some(0),
            Duration::new(20, 0),
        );
        loop {
            self.poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match TokenDec::decode(event.token()) {
                    TokenDec::ReservedToken(token) => match token {
                        SPDP_SEND_TIMER => {
                            trace!("fired SPDP_SEND_TIMER");
                            self.spdp_builtin_participant_writer
                                .write_builtin_data(data.clone());
                            self.spdp_send_timer
                                .set_timeout(CoreDuration::new(3, 0), ());
                        }
                        SPDP_PARTICIPANT_DETECTOR => {
                            while let Ok(drc) = self.spdp_builtin_participant_reader.try_recv() {
                                match drc {
                                    DataReaderStatusChanged::DataAvailable => {
                                        info!("SPDP_PARTICIPANT_DETECTOR: DataAvailable");
                                        self.handle_participant_discovery()
                                    }
                                    DataReaderStatusChanged::LivelinessChanged(_) => {
                                        info!("SPDP_PARTICIPANT_DETECTOR: LivelinessChanged");
                                    }
                                    DataReaderStatusChanged::RequestedIncompatibleQos(m) => {
                                        info!(
                                            "SPDP_PARTICIPANT_DETECTOR: RequestedIncompatibleQos('{}')",
                                            m
                                        );
                                    }
                                    DataReaderStatusChanged::SubscriptionMatched(sm) => {
                                        info!(
                                            "SPDP_PARTICIPANT_DETECTOR: SubscriptionMatched('{:?}')",
                                            sm.guid
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                        PARTICIPANT_MESSAGE_CMD_RECEIVER => {
                            while let Ok(cmd) = self.participant_msg_cmd_reveiver.try_recv() {
                                match cmd {
                                    ParticipantMessageCmd::SendData(data) => {
                                        self.p2p_builtin_participant_msg_writer.write(data);
                                    }
                                }
                            }
                        }
                        PARTICIPANT_MESSAGE_READER => { /*self.handle_participant_message()*/ }
                        PARTICIPANT_LIVELINESS_TIMER => {
                            trace!("fired PARTICIPANT_LIVELINESS_TIMER");
                            let ts = Timestamp::now().unwrap_or(Timestamp::TIME_INVALID);
                            let (next_duration, lost_participants) =
                                self.discovery_db.check_participant_liveliness(ts);
                            for l in lost_participants {
                                self.discdb_update_sender
                                    .send(DiscoveryDBUpdateNotifier::DeleteParticipant(l))
                                    .expect(
                                        "couldn't send update notification to discdb_update_sender",
                                    );
                            }
                            self.participant_liveliness_timer
                                .set_timeout(next_duration, ());
                        }
                        DISC_WRITER_ADD => {
                            while let Ok((eid, data)) = self.notify_new_writer_receiver.try_recv() {
                                self.sedp_builtin_pub_writer
                                    .write_builtin_data(data.clone());
                                self.local_writers_data.insert(eid, data);
                                info!(
                                    "add Writer to Discovery's local_writers\n\tWriter: {} ",
                                    eid
                                );
                            }
                        }
                        DISC_READER_ADD => {
                            while let Ok((eid, data)) = self.notify_new_reader_receiver.try_recv() {
                                self.sedp_builtin_sub_writer
                                    .write_builtin_data(data.clone());
                                self.local_readers_data.insert(eid, data);
                                info!(
                                    "add Reader to Discovery's local_readers\n\tReader: {} ",
                                    eid
                                );
                            }
                        }
                        Token(n) => {
                            unimplemented!("@discovery: Token(0x{:02X}) is not implemented", n)
                        }
                    },
                    TokenDec::Entity(eid) => {
                        if eid.is_reader() {
                            unimplemented!();
                        } else if eid.is_reader() {
                            unimplemented!();
                        } else {
                            unreachable!();
                        }
                    }
                }
            }
        }
    }

    fn handle_participant_discovery(&mut self) {
        let vd = self.spdp_builtin_participant_reader.take();
        info!("handle_participant_discovery: {}", vd.len());
        for mut d in vd {
            if let Some(spdp_data) = d.gen_spdp_discoverd_participant_data() {
                if spdp_data.domain_id != self.dp.domain_id() {
                    debug!(
                        "receved SPDP message from different Domain. Self: {}, Remote: {}",
                        self.dp.domain_id(),
                        spdp_data.domain_id
                    );
                    continue;
                } else {
                    let guid_prefix = spdp_data.guid.guid_prefix;
                    info!(
                        "write Participant data to discovery_db\n\tParticipant: {}",
                        spdp_data.guid
                    );
                    self.discovery_db.write_participant(
                        spdp_data.guid.guid_prefix,
                        Timestamp::now().unwrap_or(Timestamp::TIME_INVALID),
                        spdp_data,
                    );
                    self.discdb_update_sender
                        .send(DiscoveryDBUpdateNotifier::AddParticipant(guid_prefix))
                        .expect("couldn't send update notification to discdb_update_sender");
                }
            } else {
                error!("failed generate SPDPdiscoveredParticipantData from received DATA(P)");
            }
        }
    }
    /*
     * process DATA(m) which ParticipantMessageKind is MANUAL_LIVELINESS_UPDATE or AUTOMATIC_LIVELINESS_UPDATE @MessageReceiver
     * in the future, I will use this for process DATA(m) which has other ParticipantMessageKind
    fn handle_participant_message(&mut self) {
        let vd = self.p2p_builtin_participant_msg_reader.take();
        for d in vd {
            match d.kind {
                ParticipantMessageKind::MANUAL_LIVELINESS_UPDATE
                | ParticipantMessageKind::AUTOMATIC_LIVELINESS_UPDATE => {
                    let writer_guid = d.guid;
                    info!(
                        "receved DATA(m) with ParticipantMessageKind::{{MANUAL_LIVELINESS_UPDATE or AUTOMATIC_LIVELINESS_UPDATE}}",
                    );
                }
                ParticipantMessageKind::UNKNOWN => {
                    info!(
                        "receved DATA(m) with ParticipantMessageKind::UNKNOWN, which is not processed",
                    );
                }
                k => {
                    info!(
                        "receved DATA(m) with ParticipantMessageKind::{:?}, which is not processed",
                        k.value
                    );
                }
            }
        }
    }
    */
}
