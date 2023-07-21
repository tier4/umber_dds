use crate::structure::{entity::RTPSEntity, entity_id::EntityId, guid::GUID};
use mio_channel;

pub struct Writer {
    guid: GUID,
    writer_command_receiver: mio_channel::Receiver<WriterCmd>,
}

impl Writer {
    pub fn new(wi: WriterIngredients) -> Self {
        Self {
            guid: wi.guid,
            writer_command_receiver: wi.writer_command_receiver,
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
pub struct WriterCmd {}
