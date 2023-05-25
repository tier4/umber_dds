use crate::dds::{publisher::Publisher, qos::QosPolicies, topic::Topic};
use crate::rtps::writer::*;
use crate::structure::{entity::RTPSEntity, guid::GUID};
use mio_extras::channel as mio_channel;
use serde::Serialize;
use std::marker::PhantomData;

pub struct DataWriter<D: Serialize> {
    data_phantom: PhantomData<D>,
    qos: QosPolicies,
    topic: Topic,
    publisher: Publisher,
    my_guid: GUID,
    add_writer_sender: mio_channel::Sender<WriterIngredients>,
}

impl<D: Serialize> DataWriter<D> {
    pub fn new(
        add_writer_sender: mio_channel::Sender<WriterIngredients>,
        qos: QosPolicies,
        topic: Topic,
        publisher: Publisher,
    ) -> Self {
        let (writer_command_sender, writer_command_receiver) =
            mio_channel::sync_channel::<WriterCmd>(10);
        let new_writer = WriterIngredients {
            writer_command_receiver,
        };
        let my_guid = publisher.domain_participant().new_from_id();
        add_writer_sender.try_send(new_writer);
        Self {
            data_phantom: PhantomData::<D>,
            qos,
            topic,
            publisher,
            my_guid,
            add_writer_sender,
        }
    }
    pub fn get_qos() {}
    pub fn set_qos() {}
    pub fn write(&self, data: D) {}
}

impl<D: Serialize> RTPSEntity for DataWriter<D> {
    fn guid(&self) -> GUID {
        self.my_guid
    }
}
