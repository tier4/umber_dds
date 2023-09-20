use crate::message::{
    message_builder::MessageBuilder,
    message_header::Header,
    submessage::{
        element::{SequenceNumber, SerializedPayload},
        SubMessage,
    },
    Message,
};
use crate::network::udp_sender::UdpSender;
use crate::rtps::cache::{CacheChange, CacheData, ChangeKind, HistoryCache, InstantHandle};
use crate::structure::{entity::RTPSEntity, entity_id::EntityId, guid::GUID};
use bytes::Bytes;
use mio_extras::channel as mio_channel;
use mio_v06::Token;
use serde::Serialize;
use speedy::Endianness;
use std::net::Ipv4Addr;
use std::rc::Rc;

pub struct Writer {
    endianness: Endianness,
    guid: GUID,
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
    writer_cache: HistoryCache,
    lastChangeSequenceNumber: SequenceNumber,
    sender: Rc<UdpSender>,
}

impl Writer {
    pub fn new(wi: WriterIngredients, sender: Rc<UdpSender>) -> Self {
        Self {
            endianness: Endianness::LittleEndian,
            guid: wi.guid,
            writer_command_receiver: wi.writer_command_receiver,
            writer_cache: HistoryCache::new(),
            lastChangeSequenceNumber: SequenceNumber(0),
            sender,
        }
    }

    pub fn entity_token(&self) -> Token {
        self.entity_id().as_token()
    }

    pub fn handle_writer_cmd(&mut self) {
        loop {
            let cmd = match self.writer_command_receiver.try_recv() {
                Ok(c) => {
                    // this is new_change
                    self.lastChangeSequenceNumber += SequenceNumber(1);
                    let cache_data = match &c.serialized_payload {
                        Some(v) => CacheData::new(v.value.clone()),
                        None => CacheData::new(Bytes::from("")),
                    };
                    let a_change = CacheChange::new(
                        ChangeKind::Alive,
                        self.guid,
                        self.lastChangeSequenceNumber,
                        cache_data,
                        InstantHandle {},
                    );
                    // TODO: register a_change to writer HistoryCache
                    // build RTPS Message
                    let mut message_builder = MessageBuilder::new();
                    message_builder.data(
                        Endianness::LittleEndian,
                        EntityId::UNKNOW,
                        self.guid.entity_id,
                        a_change,
                        c.serialized_payload,
                    );
                    let message = message_builder.build(self.guid_prefix());
                    let message_buf = todo!(); // TODO: impl Message for serializer
                    self.sender
                        .send_to_multicast(message_buf, Ipv4Addr::new(239, 255, 0, 1), 7400);
                }
                Err(_) => break,
            };
            // println!("writer received : {:?}", cmd.serialized_data);
        }
    }
}

impl RTPSEntity for Writer {
    fn guid(&self) -> GUID {
        self.guid
    }
}

pub struct WriterIngredients {
    pub guid: GUID,
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
}
pub struct WriterCmd {
    pub serialized_payload: Option<SerializedPayload>,
}
