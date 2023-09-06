use crate::message::{
    message_header::Header,
    submessage::{element::SerializedPayload, SubMessage},
    Message,
};
use crate::network::udp_sender::UdpSender;
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
    sender: Rc<UdpSender>,
}

impl Writer {
    pub fn new(wi: WriterIngredients, sender: Rc<UdpSender>) -> Self {
        Self {
            endianness: Endianness::LittleEndian,
            guid: wi.guid,
            writer_command_receiver: wi.writer_command_receiver,
            sender,
        }
    }

    pub fn entity_token(&self) -> Token {
        self.entity_id().as_token()
    }

    pub fn handle_writer_cmd(&self) {
        loop {
            let cmd = match self.writer_command_receiver.try_recv() {
                Ok(c) => {
                    // TODO: build RTPS Message
                    let message_header = Header::new(self.guid_prefix());
                    let mut submessages: Vec<SubMessage> = Vec::new();
                    self.sender.send_to_multicast(
                        &c.serialized_payload,
                        Ipv4Addr::new(239, 255, 0, 1),
                        7400,
                    );
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
    pub serialized_payload: Bytes,
}
