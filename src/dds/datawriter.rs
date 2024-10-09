use crate::dds::{
    publisher::Publisher,
    qos::{policy::LivelinessQosKind, DataWriterQosPolicies},
    topic::Topic,
};
use crate::message::submessage::element::{RepresentationIdentifier, SerializedPayload};
use crate::rtps::writer::*;
use core::marker::PhantomData;
use mio_extras::channel as mio_channel;
use serde::Serialize;

/// DDS DataWriter
#[allow(dead_code)]
pub struct DataWriter<D: Serialize> {
    data_phantom: PhantomData<D>,
    qos: DataWriterQosPolicies,
    topic: Topic,
    publisher: Publisher,
    // my_guid: GUID, // In RustDDS, DataWriter has guid to drop corresponding RTPSWriter
    // I implement guid for DataWriter when need.
    writer_command_sender: mio_channel::SyncSender<WriterCmd>,
}

impl<D: Serialize> DataWriter<D> {
    pub(crate) fn new(
        writer_command_sender: mio_channel::SyncSender<WriterCmd>,
        qos: DataWriterQosPolicies,
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
    pub fn get_qos(&self) -> DataWriterQosPolicies {
        self.qos.clone()
    }
    pub fn set_qos(&mut self, qos: DataWriterQosPolicies) {
        self.qos = qos;
    }

    /// publish data for matching DataWriter
    pub fn write(&self, data: D) {
        let serialized_payload =
            SerializedPayload::new_from_cdr_data(data, RepresentationIdentifier::CDR_LE);
        let writer_cmd = WriterCmd::WriteData(Some(serialized_payload));
        self.writer_command_sender
            .send(writer_cmd)
            .expect("couldn't send message");
    }

    pub(crate) fn write_builtin_data(&self, data: D) {
        let serialized_payload =
            SerializedPayload::new_from_cdr_data(data, RepresentationIdentifier::PL_CDR_LE);
        let writer_cmd = WriterCmd::WriteData(Some(serialized_payload));
        self.writer_command_sender
            .send(writer_cmd)
            .expect("couldn't send message");
    }

    pub fn assert_liveliness(&self) -> Result<(), String> {
        match self.qos.liveliness.kind {
            LivelinessQosKind::Automatic => Ok(()),
            LivelinessQosKind::ManualByParticipant => {
                unimplemented!(
                    "behavior of LivelinessQosKind::ManualByParticipant is not yet implemented"
                );
                let writer_cmd = WriterCmd::AssertLiveliness();
                self.writer_command_sender
                    .send(writer_cmd)
                    .expect("couldn't send message");
            }
            LivelinessQosKind::ManualByTopic => {
                unimplemented!(
                    "behavior of LivelinessQosKind::ManualByTopic is not yet implemented"
                );
                let writer_cmd = WriterCmd::AssertLiveliness();
                self.writer_command_sender
                    .send(writer_cmd)
                    .expect("couldn't send message");
            }
        }
    }
}
