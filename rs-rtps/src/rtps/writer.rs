use crate::structure::{entity::RTPSEntity, entity_id::EntityId, guid::GUID};
use bytes::Bytes;
use mio::Token;
use mio_channel;
use serde::Serialize;

pub struct Writer {
    guid: GUID,
    pub writer_command_receiver: mio_channel::Receiver<WriterCmd>,
}

impl Writer {
    pub fn new(wi: WriterIngredients) -> Self {
        Self {
            guid: wi.guid,
            writer_command_receiver: wi.writer_command_receiver,
        }
    }

    pub fn entity_token(&self) -> Token {
        self.entity_id().as_token()
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
    pub serialized_data: Bytes,
}
