use crate::dds::{
    datawriter::DataWriter,
    participant::DomainParticipant,
    publisher::Publisher,
    qos::{QosBuilder, QosPolicies},
    tokens::*,
    topic::Topic,
    typedesc::TypeDesc,
};
use crate::discovery::structure::data::SPDPdiscoveredParticipantData;
use crate::message::{
    message_header::ProtocolVersion,
    submessage::element::{Count, Locator},
};
use crate::network::net_util::{
    spdp_multicast_port, spdp_unicast_port, usertraffic_multicast_port, usertraffic_unicast_port,
};
use crate::structure::{
    entity::RTPSEntity, entity_id::EntityId, topic_kind::TopicKind, vendor_id::VendorId,
};
use mio_extras::{channel as mio_channel, timer::Timer};
use mio_v06::net::UdpSocket;
use mio_v06::{Events, Poll, PollOpt, Ready, Token};
use std::time::Duration;

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
    spdp_send_timer: Timer<()>,
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
        let mut spdp_send_timer: Timer<()> = Timer::default();
        spdp_send_timer.set_timeout(Duration::new(3, 0), ());
        poll.register(
            &mut spdp_send_timer,
            SPDP_SEND_TIMER,
            Ready::readable(),
            PollOpt::edge(),
        )
        .unwrap();
        Self {
            dp,
            poll,
            publisher,
            spdp_builtin_participant_writer,
            spdp_send_timer,
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
            0x18000c3f,
            Locator::new_list_from_self_ipv4(spdp_unicast_port(domain_id, participant_id) as u32),
            Locator::new_list_from_self_ipv4(spdp_multicast_port(domain_id) as u32),
            Locator::new_list_from_self_ipv4(
                usertraffic_unicast_port(domain_id, participant_id) as u32
            ),
            Locator::new_list_from_self_ipv4(usertraffic_multicast_port(domain_id) as u32),
            0,
            crate::structure::duration::Duration::new(20, 0),
        );
        loop {
            self.poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match TokenDec::decode(event.token()) {
                    TokenDec::ReservedToken(token) => match token {
                        SPDP_SEND_TIMER => {
                            eprintln!("@discovery: timer timedout");
                            self.spdp_builtin_participant_writer
                                .write_builtin_data(data.clone());
                            self.spdp_send_timer.set_timeout(Duration::new(3, 0), ());
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
}
