use crate::discovery::discovery_db::DiscoveryDB;
use crate::discovery::structure::{
    cdr::deserialize,
    data::{SDPBuiltinData, SPDPdiscoveredParticipantData},
};
use crate::message::{
    submessage::{element::*, submessage_flag::*, *},
    *,
};
use crate::net_util::*;
use crate::rtps::{
    cache::{CacheChange, CacheData, ChangeKind, InstantHandle},
    reader::Reader,
    writer::Writer,
};
use crate::structure::entity_id::EntityId;
use crate::structure::{guid::*, vendor_id::*};
use colored::*;
use mio_extras::channel as mio_channel;
use std::collections::HashMap;

pub struct MessageReceiver {
    own_guid_prefix: GuidPrefix,
    source_version: ProtocolVersion,
    source_vendor_id: VendorId,
    source_guid_prefix: GuidPrefix,
    dest_guid_prefix: GuidPrefix,
    unicast_reply_locator_list: Vec<Locator>,
    multicast_reply_locator_list: Vec<Locator>,
    have_timestamp: bool,
    timestamp: Timestamp,
    discovery_db: DiscoveryDB,
    discdb_update_sender: mio_channel::Sender<GuidPrefix>,
}

impl MessageReceiver {
    pub fn new(
        participant_guidprefix: GuidPrefix,
        discovery_db: DiscoveryDB,
        discdb_update_sender: mio_channel::Sender<GuidPrefix>,
    ) -> MessageReceiver {
        Self {
            own_guid_prefix: participant_guidprefix,
            source_version: ProtocolVersion::PROTOCOLVERSION,
            source_vendor_id: VendorId::VENDORID_UNKNOW,
            source_guid_prefix: GuidPrefix::UNKNOW,
            dest_guid_prefix: GuidPrefix::UNKNOW,
            unicast_reply_locator_list: vec![Locator::INVALID],
            multicast_reply_locator_list: vec![Locator::INVALID],
            have_timestamp: false,
            timestamp: Timestamp::TIME_INVALID,
            discovery_db,
            discdb_update_sender,
        }
    }

    fn reset(&mut self) {
        self.source_version = ProtocolVersion::PROTOCOLVERSION;
        self.source_vendor_id = VendorId::VENDORID_UNKNOW;
        self.source_guid_prefix = GuidPrefix::UNKNOW;
        self.dest_guid_prefix = GuidPrefix::UNKNOW;
        self.unicast_reply_locator_list.clear();
        self.multicast_reply_locator_list.clear();
        self.have_timestamp = false;
        self.timestamp = Timestamp::TIME_INVALID;
    }

    pub fn handle_packet(
        &mut self,
        messages: Vec<UdpMessage>,
        mut writers: &mut HashMap<EntityId, Writer>,
        mut readers: &mut HashMap<EntityId, Reader>,
    ) {
        for message in messages {
            // Is DDSPING
            let msg = message.message;
            if msg.len() < 20 {
                if msg.len() >= 16 && msg[0..4] == b"RTPS"[..] && msg[9..16] == b"DDSPING"[..] {
                    println!("Received DDSPING");
                    return;
                }
            }
            let msg_buf = msg.freeze();
            let rtps_message = match Message::new(msg_buf) {
                Ok(m) => m,
                Err(e) => {
                    println!("{}: couldn't deserialize : {}", "error".red(), e);
                    return;
                }
            };
            if rtps_message.header.guid_prefix == self.own_guid_prefix {
                eprintln!("*****  RTPS message form self. *****");
                return;
            }
            eprintln!("self.own_guid_prefix: {:?}", self.own_guid_prefix);
            eprintln!(
                "*****  RTPS message form {:?}. *****",
                rtps_message.header.guid_prefix
            );
            self.handle_parsed_packet(rtps_message, &mut writers, &mut readers);
        }
    }

    fn handle_parsed_packet(
        &mut self,
        rtps_msg: Message,
        mut writers: &mut HashMap<EntityId, Writer>,
        mut readers: &mut HashMap<EntityId, Reader>,
    ) {
        self.reset();
        self.dest_guid_prefix = self.own_guid_prefix;
        self.source_guid_prefix = rtps_msg.header.guid_prefix;
        println!(">>>>>>>>>>>>>>>>");
        println!("header: {:?}", rtps_msg.header);
        for submsg in rtps_msg.submessages {
            println!("submessage header: {:?}", submsg.header);
            println!("submessage kind: {:?}", submsg.header.get_submessagekind());
            match submsg.body {
                SubMessageBody::Entity(e) => {
                    self.handle_entity_submessage(e, &mut writers, &mut readers)
                }
                SubMessageBody::Interpreter(i) => self.handle_interpreter_submessage(i),
            }
        }
        println!("<<<<<<<<<<<<<<<<\n");
    }

    fn handle_entity_submessage(
        &mut self,
        entity_subm: EntitySubmessage,
        mut writers: &mut HashMap<EntityId, Writer>,
        mut readers: &mut HashMap<EntityId, Reader>,
    ) {
        println!("handle entity submsg");
        match entity_subm {
            EntitySubmessage::AckNack(acknack, flags) => {
                self.handle_acknack_submsg(acknack, flags, &mut writers)
            }
            EntitySubmessage::Data(data, flags) => {
                self.handle_data_submsg(data, flags, &mut readers)
            }
            EntitySubmessage::DataFrag(dat_frag, flags) => (),
            EntitySubmessage::Gap(gap, flags) => (),
            EntitySubmessage::HeartBeat(heartbeat, flags) => (),
            EntitySubmessage::HeartbeatFrag(heartbeatfrag, flags) => (),
            EntitySubmessage::NackFrag(nack_frag, flags) => (),
        }
    }
    fn handle_interpreter_submessage(&mut self, interpreter_subm: InterpreterSubmessage) {
        println!("handle interpreter submsg");
        match interpreter_subm {
            InterpreterSubmessage::InfoReply(info_reply, flags) => {
                self.unicast_reply_locator_list = info_reply.unicast_locator_list;
                if flags.contains(InfoReplyFlag::Multicast) {
                    self.multicast_reply_locator_list = info_reply
                        .multicast_locator_list
                        .expect("invalid InfoReply");
                } else {
                    self.multicast_reply_locator_list.clear();
                }
            }
            InterpreterSubmessage::InfoReplyIp4(info_reply_ip4, flags) => {
                self.unicast_reply_locator_list = info_reply_ip4.unicast_locator_list;
                if flags.contains(InfoReplyIp4Flag::Multicast) {
                    self.multicast_reply_locator_list = info_reply_ip4
                        .multicast_locator_list
                        .expect("invalid InfoReplyIp4");
                } else {
                    self.multicast_reply_locator_list.clear();
                }
            }
            InterpreterSubmessage::InfoTimestamp(info_ts, flags) => {
                if !flags.contains(InfoTimestampFlag::Invalidate) {
                    self.have_timestamp = true;
                    self.timestamp = info_ts.timestamp.expect("invalid InfoTS");
                } else {
                    self.have_timestamp = false;
                }
            }
            InterpreterSubmessage::InfoSource(info_souce, _flags) => {
                self.source_guid_prefix = info_souce.guid_prefix;
                self.source_version = info_souce.protocol_version;
                self.source_vendor_id = info_souce.vendor_id;
                self.unicast_reply_locator_list.clear();
                self.multicast_reply_locator_list.clear();
                self.have_timestamp = false;
            }
            InterpreterSubmessage::InfoDestination(info_dst, _flags) => {
                if info_dst.guid_prefix != GuidPrefix::UNKNOW {
                    self.dest_guid_prefix = info_dst.guid_prefix;
                } else {
                    self.dest_guid_prefix = self.own_guid_prefix;
                }
            }
        }
    }

    fn handle_acknack_submsg(
        &self,
        ackanck: AckNack,
        flag: BitFlags<AckNackFlag>,
        writers: &mut HashMap<EntityId, Writer>,
    ) {
        // validation
        if !ackanck.reader_sn_state.validate() {
            panic!("Invalid AckNack Submessage received");
        }
        let writer_guid = GUID::new(self.dest_guid_prefix, ackanck.writer_id);
        let reader_guid = GUID::new(self.source_guid_prefix, ackanck.reader_id);
        // TODO: writer.handle_acknackに適切な引数を渡す。
        match writers.get_mut(&ackanck.writer_id) {
            Some(w) => w.handle_acknack(),
            None => (),
        }
    }
    fn handle_data_submsg(
        &mut self,
        data: Data,
        flag: BitFlags<DataFlag>,
        readers: &mut HashMap<EntityId, Reader>,
    ) {
        // validation
        if data.writer_sn < SequenceNumber(0)
            || data.writer_sn == SequenceNumber::SEQUENCENUMBER_UNKNOWN
        {
            panic!("Invalid Data Submessage received");
        }
        // TODO: check inlineQos is valid
        if flag.contains(DataFlag::Data) && !flag.contains(DataFlag::Key) {
            // he serializedPayload element is interpreted as the value of the dtat-object
        }
        if flag.contains(DataFlag::Key) && !flag.contains(DataFlag::Data) {
            // the serializedPayload element is interpreted as the value of the key that identifies the registered instance of the data-object.
        }
        if flag.contains(DataFlag::InlineQos) {
            // the inlineQos element contains QoS values that override those of the RTPS Writer and should
            // be used to process the update. For a complete list of possible in-line QoS parameters, see Table 8.80.
        }
        if flag.contains(DataFlag::NonStandardPayload) {
            // the serializedPayload element is not formatted according to Section 10.
            // This flag is informational. It indicates that the SerializedPayload has been transformed as described in another specification
            // For example, this flag should be set when the SerializedPayload is transformed as described in the DDS-Security specification
        }
        let writer_guid = GUID::new(self.dest_guid_prefix, data.writer_id);
        let reader_guid = GUID::new(self.source_guid_prefix, data.reader_id);

        // if msg is for SPDP
        if data.writer_id == EntityId::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER {
            let mut deserialized = match deserialize::<SDPBuiltinData>(
                &data.serialized_payload.as_ref().unwrap().to_bytes(),
            ) {
                Ok(d) => d,
                Err(e) => panic!(
                    "neko~~~~~: failed deserialize reseived spdp data message: {}",
                    e
                ),
            };
            let new_data = deserialized.toSPDPdiscoverdParticipantData();
            let timestamp = Timestamp::now().expect("Couldn't get timestamp");
            let guid_prefix = new_data.guid.guid_prefix;
            self.discovery_db.write(guid_prefix, timestamp, new_data);
            self.discdb_update_sender.send(guid_prefix).unwrap();
            println!(
                "##################  @message_receiver  Discovery message received from: {:?}",
                guid_prefix
            );
            /*
            eprintln!("domain_id: {}", new_data.domain_id);
            eprintln!("domain_tag: {}", new_data.domain_tag);
            eprintln!("protocol_version: {:?}", new_data.protocol_version);
            eprintln!("guid: {:?}", new_data.guid);
            eprintln!("vendor_id: {:?}", new_data.vendor_id);
            eprintln!("expects_inline_qos: {:?}", new_data.expects_inline_qos);
            eprintln!(
                "available_builtin_endpoint: {:?}",
                new_data.available_builtin_endpoint
            );
            eprintln!(
                "metarraffic_unicast_locator_list: {:?}",
                new_data.metarraffic_unicast_locator_list
            );
            eprintln!(
                "metarraffic_multicast_locator_list: {:?}",
                new_data.metarraffic_multicast_locator_list
            );
            eprintln!(
                "default_unicast_locator_list: {:?}",
                new_data.default_unicast_locator_list
            );
            eprintln!(
                "default_multicast_locator_list: {:?}",
                new_data.default_multicast_locator_list
            );
            eprintln!(
                "manual_liveliness_count: {:?}",
                new_data.manual_liveliness_count
            );
            eprintln!("lease_duration: {:?}", new_data.lease_duration.clone());
            */
        }
        // if msg is for SEDP
        if data.writer_id == EntityId::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER {
            let mut deserialized = match deserialize::<SDPBuiltinData>(
                &data.serialized_payload.as_ref().unwrap().to_bytes(),
            ) {
                Ok(d) => d,
                Err(e) => panic!(
                    "neko~~~~~: failed deserialize reseived sedp(w) data message: {}",
                    e
                ),
            };
            eprintln!("successed for deserialize sedp(w)");
        }
        // if msg is for SEDP
        if data.writer_id == EntityId::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER {
            let mut deserialized = match deserialize::<SDPBuiltinData>(
                &data.serialized_payload.as_ref().unwrap().to_bytes(),
            ) {
                Ok(d) => d,
                Err(e) => panic!(
                    "neko~~~~~: failed deserialize reseived sedp(r) data message: {}",
                    e
                ),
            };
            eprintln!("successed for deserialize sedp(r)");
        }
        let cache_data = match data.serialized_payload {
            Some(s) => Some(CacheData::new(s.value)),
            None => None,
        };

        let change = CacheChange::new(
            ChangeKind::Alive,
            writer_guid,
            data.writer_sn,
            cache_data,
            InstantHandle {}, // TODO
        );
        match readers.get_mut(&data.reader_id) {
            Some(r) => r.add_change(change),
            None => (),
        }
    }
    fn handle_datafrag_submsg(data_frag: DataFrag, flag: BitFlags<DataFragFlag>) {}
    fn handle_gap_submsg(gap: Gap, flag: BitFlags<GapFlag>) {}
    fn handle_heartbeat_submsg(heartbeat: Heartbeat, flag: BitFlags<HeartbeatFlag>) {}
    fn handle_heartbeatfrag_submsg(
        heartbeat_frag: HeartbeatFrag,
        flag: BitFlags<HeartbeatFragFlag>,
    ) {
    }
    fn handle_nackfrag_submsg(nack_frag: NackFrag, flag: BitFlags<NackFragFlag>) {}
}
