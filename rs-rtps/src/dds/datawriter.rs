use crate::dds::{publisher::Publisher, qos::QosPolicies, topic::Topic};
use crate::rtps::writer::*;
use crate::structure::{entity::RTPSEntity, entity_id::EntityId, guid::GUID};
use mio_channel;
use serde::Serialize;
use std::marker::PhantomData;

pub struct DataWriter<D: Serialize> {
    data_phantom: PhantomData<D>,
    qos: QosPolicies,
    topic: Topic,
    publisher: Publisher,
    // my_guid: GUID, // In RustDDS, DataWriter has guid to drop corresponding RTPSWriter
    // I implement guid for DataWriter when need.
    add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
}

impl<D: Serialize> DataWriter<D> {
    pub fn new(
        add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
        qos: QosPolicies,
        topic: Topic,
        publisher: Publisher,
    ) -> Self {
        let (writer_command_sender, writer_command_receiver) =
            mio_channel::sync_channel::<WriterCmd>(10);
        let new_writer = WriterIngredients {
            writer_command_receiver,
        };
        add_writer_sender.send(new_writer);
        Self {
            data_phantom: PhantomData::<D>,
            qos,
            topic,
            publisher,
            add_writer_sender,
        }
    }
    pub fn get_qos() {}
    pub fn set_qos() {}
    pub fn write(&self, data: D) {}
}
