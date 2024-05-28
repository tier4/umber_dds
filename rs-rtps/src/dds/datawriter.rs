use crate::dds::{publisher::Publisher, qos::QosPolicies, topic::Topic};
use crate::message::submessage::element::{RepresentationIdentifier, SerializedPayload};
use crate::rtps::writer::*;
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
        let serialized_payload =
            SerializedPayload::new_from_cdr_data(data, RepresentationIdentifier::CDR_LE);
        let writer_cmd = WriterCmd {
            serialized_payload: Some(serialized_payload),
        };
        self.writer_command_sender
            .send(writer_cmd)
            .expect("couldn't send message");
    }

    pub fn write_builtin_data(&self, data: D) {
        let serialized_payload =
            SerializedPayload::new_from_cdr_data(data, RepresentationIdentifier::PL_CDR_LE);
        let writer_cmd = WriterCmd {
            serialized_payload: Some(serialized_payload),
        };
        self.writer_command_sender
            .send(writer_cmd)
            .expect("couldn't send message");
    }
}
