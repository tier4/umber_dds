use crate::dds::{
    datareader::DataReader,
    datawriter::DataWriter,
    participant::DomainParticipant,
    publisher::Publisher,
    qos::{
        policy::*, DataReadedrQos, DataReadedrQosBuilder, DataWriterQos, DataWriterQosBuilder,
        PublisherQos, PublisherQosBuilder, SubscriberQos, SubscriberQosBuilder, TopicQos,
        TopicQosBuilder,
    },
    subscriber::Subscriber,
    tokens::*,
    topic::Topic,
};
use crate::discovery::discovery_db::DiscoveryDB;
use crate::discovery::structure::builtin_endpoint::BuiltinEndpoint;
use crate::discovery::structure::data::{
    DiscoveredReaderData, DiscoveredWriterData, SDPBuiltinData, SPDPdiscoveredParticipantData,
};
use crate::message::{
    message_header::ProtocolVersion,
    submessage::element::{Locator, Timestamp},
};
use crate::network::net_util::{
    spdp_multicast_port, spdp_unicast_port, usertraffic_multicast_port, usertraffic_unicast_port,
};
use crate::structure::{
    duration::Duration, entity::RTPSEntity, entity_id::EntityId, guid::GuidPrefix,
    topic_kind::TopicKind, vendor_id::VendorId,
};
use alloc::collections::BTreeMap;
use colored::*;
use enumflags2::make_bitflags;
use mio_extras::{channel as mio_channel, timer::Timer};
use mio_v06::{Events, Poll, PollOpt, Ready, Token};
use std::time::Duration as StdDuration;

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

pub struct Discovery {
    dp: DomainParticipant,
    discovery_db: DiscoveryDB,
    discdb_update_sender: mio_channel::Sender<GuidPrefix>,
    poll: Poll,
    publisher: Publisher,
    subscriber: Subscriber,
    spdp_builtin_participant_writer: DataWriter<SPDPdiscoveredParticipantData>,
    spdp_builtin_participant_reader: DataReader<SDPBuiltinData>,
    sedp_builtin_pub_writer: DataWriter<DiscoveredWriterData>,
    sedp_builtin_pub_reader: DataReader<SDPBuiltinData>,
    sedp_builtin_sub_writer: DataWriter<DiscoveredReaderData>,
    sedp_builtin_sub_reader: DataReader<SDPBuiltinData>,
    spdp_send_timer: Timer<()>,
    writers_data: BTreeMap<EntityId, DiscoveredWriterData>,
    writer_add_receiver: mio_channel::Receiver<(EntityId, DiscoveredWriterData)>,
    readers_data: BTreeMap<EntityId, DiscoveredReaderData>,
    reader_add_receiver: mio_channel::Receiver<(EntityId, DiscoveredReaderData)>,
}

impl Discovery {
    pub fn new(
        dp: DomainParticipant,
        discovery_db: DiscoveryDB,
        discdb_update_sender: mio_channel::Sender<GuidPrefix>,
        mut writer_add_receiver: mio_channel::Receiver<(EntityId, DiscoveredWriterData)>,
        mut reader_add_receiver: mio_channel::Receiver<(EntityId, DiscoveredReaderData)>,
    ) -> Self {
        let poll = Poll::new().unwrap();
        let publisher = dp.create_publisher(PublisherQos::Default);
        let subscriber = dp.create_subscriber(SubscriberQos::Default);

        // For SPDP
        let spdp_topic = dp.create_topic(
            "DCPSParticipant".to_string(),
            "SPDPDiscoveredParticipantData".to_string(),
            TopicKind::WithKey,
            TopicQos::Default,
        );
        let spdp_writer_qos = DataWriterQos::Policies(
            DataWriterQosBuilder::new()
                .reliability(Reliability::default_besteffort())
                .build(),
        );
        let spdp_reader_qos = DataReadedrQos::Policies(
            DataReadedrQosBuilder::new()
                .reliability(Reliability::default_besteffort())
                .build(),
        );
        let spdp_writer_entity_id = EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;
        let spdp_reader_entity_id = EntityId::SPDP_BUILTIN_PARTICIPANT_DETECTOR;
        let spdp_builtin_participant_writer = publisher.create_datawriter_with_entityid(
            spdp_writer_qos,
            spdp_topic.clone(),
            spdp_writer_entity_id,
        );
        let mut spdp_builtin_participant_reader = subscriber.create_datareader_with_entityid(
            spdp_reader_qos,
            spdp_topic.clone(),
            spdp_reader_entity_id,
        );
        poll.register(
            &mut spdp_builtin_participant_reader,
            SPDP_PARTICIPANT_DETECTOR,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("couldn't register spdp_builtin_participant_reader to poll");

        // For SEDP
        let sedp_writer_qos = DataWriterQos::Policies(
            DataWriterQosBuilder::new()
                .reliability(Reliability::default_reliable())
                .build(),
        );
        let sedp_reader_qos = DataReadedrQos::Policies(
            DataReadedrQosBuilder::new()
                .reliability(Reliability::default_reliable())
                .build(),
        );
        let sedp_topic_qos = TopicQos::Policies(
            TopicQosBuilder::new()
                .reliability(Reliability::default_reliable())
                .build(),
        );
        let sedp_publication_topic = dp.create_topic(
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
        let sedp_subscription_topic = dp.create_topic(
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

        let mut spdp_send_timer: Timer<()> = Timer::default();
        spdp_send_timer.set_timeout(StdDuration::new(3, 0), ());
        poll.register(
            &mut spdp_send_timer,
            SPDP_SEND_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("couldn't register spdp_send_timer to poll");
        poll.register(
            &mut writer_add_receiver,
            DISC_WRITER_ADD,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("couldn't register writer_add_receiver to poll");
        poll.register(
            &mut reader_add_receiver,
            DISC_READER_ADD,
            Ready::readable(),
            PollOpt::edge(),
        )
        .expect("couldn't register reader_add_receiver to poll");
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
            spdp_send_timer,
            writers_data: BTreeMap::new(),
            writer_add_receiver,
            readers_data: BTreeMap::new(),
            reader_add_receiver,
        }
    }

    pub fn discovery_loop(&mut self) {
        let mut events = Events::with_capacity(1024);
        let domain_id = self.dp.domain_id();
        let participant_id = self.dp.participant_id();
        // This data is for test
        let data = SPDPdiscoveredParticipantData::new(
            self.dp.domain_id(),
            String::from("todo"),
            ProtocolVersion::PROTOCOLVERSION,
            self.dp.guid(),
            VendorId::THIS_IMPLEMENTATION,
            false,
            make_bitflags!(BuiltinEndpoint::{DISC_BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR}),
            Locator::new_list_from_self_ipv4(spdp_unicast_port(domain_id, participant_id) as u32),
            vec![Locator::new_from_ipv4(
                spdp_multicast_port(domain_id) as u32,
                [239, 255, 0, 1],
            )],
            Locator::new_list_from_self_ipv4(
                usertraffic_unicast_port(domain_id, participant_id) as u32
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
                            self.spdp_builtin_participant_writer
                                .write_builtin_data(data.clone());
                            self.spdp_send_timer.set_timeout(StdDuration::new(3, 0), ());
                        }
                        SPDP_PARTICIPANT_DETECTOR => self.handle_participant_discovery(),
                        DISC_WRITER_ADD => {
                            while let Ok((eid, data)) = self.writer_add_receiver.try_recv() {
                                self.sedp_builtin_pub_writer
                                    .write_builtin_data(data.clone());
                                self.writers_data.insert(eid, data);
                                eprintln!(
                                    "<{}>: add writer which has {:?} to writers",
                                    "Discovery: Info".green(),
                                    eid
                                );
                            }
                        }
                        DISC_READER_ADD => {
                            while let Ok((eid, data)) = self.reader_add_receiver.try_recv() {
                                self.sedp_builtin_sub_writer
                                    .write_builtin_data(data.clone());
                                self.readers_data.insert(eid, data);
                                eprintln!(
                                    "<{}>: add reader which has {:?} to readers",
                                    "Discovery: Info".green(),
                                    eid
                                );
                            }
                        }
                        Token(n) => unimplemented!("@discovery: Token({}) is not implemented", n),
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
        for mut d in vd {
            if let Some(spdp_data) = d.to_spdp_discoverd_participant_data() {
                if spdp_data.domain_id != self.dp.domain_id() {
                    continue;
                } else {
                    let guid_prefix = spdp_data.guid.guid_prefix;
                    eprintln!("<{}>: discovery_db wite", "Discovery: Info".green());
                    self.discovery_db.write(
                        spdp_data.guid.guid_prefix,
                        Timestamp::now().unwrap_or(Timestamp::TIME_INVALID),
                        spdp_data,
                    );
                    self.discdb_update_sender
                        .send(guid_prefix)
                        .expect("couldn't send update notification to discdb_update_sender");
                }
            }
        }
    }
}