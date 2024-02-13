use crate::dds::{
    datawriter::DataWriter,
    participant::DomainParticipant,
    publisher::Publisher,
    qos::{QosBuilder, QosPolicies},
    topic::Topic,
    typedesc::TypeDesc,
};
use crate::discovery::structure::data::SPDPdiscoveredParticipantData;
use crate::structure::{entity_id::EntityId, topic_kind::TopicKind};
use mio_extras::{channel as mio_channel, timer::Timer};
use mio_v06::net::UdpSocket;
use mio_v06::{Events, Poll, PollOpt, Ready, Token};

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

pub struct Discovery {
    dp: DomainParticipant,
    poll: Poll,
    publisher: Publisher,
    spdp_builtin_participant_writer: DataWriter<SPDPdiscoveredParticipantData>,
}

impl Discovery {
    pub fn new(dp: DomainParticipant) -> Self {
        let poll = Poll::new().unwrap();
        let qos = QosBuilder::new().build();
        let publisher = dp.create_publisher(qos);
        let topic = Topic::new(
            "DCPSParticipant".to_string(),
            TypeDesc::new("SPDPDiscoveredParticipantData".to_string()),
            dp.clone(),
            qos,
            TopicKind::WithKey,
        );
        let entity_id = EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;
        let spdp_builtin_participant_writer =
            publisher.create_datawriter_with_entityid(qos, topic, entity_id);
        Self {
            dp,
            poll,
            publisher,
            spdp_builtin_participant_writer,
        }
    }

    pub fn discovery_loop() {
        loop {}
    }
}
