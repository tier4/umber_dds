use crate::dds::{publisher::Publisher, qos::QosPolicies, topic::Topic};
use crate::rtps::writer::*;
use crate::structure::{entity::RTPSEntity, entity_id::EntityId, guid::GUID};
use mio_extras::channel as mio_channel;
use serde::Serialize;
use std::marker::PhantomData;

pub struct DataWriter<D: Serialize> {
    data_phantom: PhantomData<D>,
    qos: QosPolicies,
    topic: Topic,
    publisher: Publisher,
    // my_guid: GUID, // In RustDDS, DataWriter has guid to drop corresponding RTPSWriter
    // I implement guid for DataWriter when need.
    writer_command_sender: mio_channel::SyncSender<WriterCmd>,
}

impl<D: Serialize> DataWriter<D> {
    pub fn new(
        writer_command_sender: mio_channel::SyncSender<WriterCmd>,
        qos: QosPolicies,
        topic: Topic,
        publisher: Publisher,
    ) -> Self {
        Self {
            data_phantom: PhantomData::<D>,
            qos,
            topic,
            publisher,
            writer_command_sender,
        }
    }
    pub fn get_qos() {}
    pub fn set_qos() {}
    pub fn write(&self, data: D) {
        // TODO: serialize data to Bytes
        // RTPS spec: 10 Serialized Payload Representation
        // https://www.omg.org/spec/DDSI-RTPS/2.3/Beta1/PDF#%5B%7B%22num%22%3A559%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C46%2C756%2C0%5D
        let serialized_data = bytes::Bytes::from("dummy data");
        let writer_cmd = WriterCmd { serialized_data };
        self.writer_command_sender.send(writer_cmd).unwrap();
    }
}
