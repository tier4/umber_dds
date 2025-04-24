use crate::dds::{
    key::DdsData,
    publisher::Publisher,
    qos::{policy::LivelinessQosKind, DataWriterQosPolicies},
    topic::Topic,
};
use crate::message::submessage::element::{RepresentationIdentifier, SerializedPayload};
use crate::rtps::writer::*;
use core::marker::PhantomData;
use mio_extras::channel as mio_channel;
use mio_v06::{event::Evented, Poll, PollOpt, Ready, Token};
use serde::Serialize;
use std::io;

/// DDS DataWriter
#[allow(dead_code)]
pub struct DataWriter<D: Serialize + DdsData> {
    data_phantom: PhantomData<D>,
    qos: DataWriterQosPolicies,
    topic: Topic,
    publisher: Publisher,
    // my_guid: GUID, // In RustDDS, DataWriter has guid to drop corresponding RTPSWriter
    // I implement guid for DataWriter when need.
    writer_command_sender: mio_channel::SyncSender<WriterCmd>,
    writer_state_receiver: mio_channel::Receiver<DataWriterStatusChanged>,
}

impl<D: Serialize + DdsData> DataWriter<D> {
    pub(crate) fn new(
        writer_command_sender: mio_channel::SyncSender<WriterCmd>,
        qos: DataWriterQosPolicies,
        topic: Topic,
        publisher: Publisher,
        writer_state_receiver: mio_channel::Receiver<DataWriterStatusChanged>,
    ) -> Self {
        Self {
            data_phantom: PhantomData::<D>,
            qos,
            topic,
            publisher,
            writer_command_sender,
            writer_state_receiver,
        }
    }
    pub fn get_qos(&self) -> DataWriterQosPolicies {
        self.qos.clone()
    }
    pub fn set_qos(&mut self, qos: DataWriterQosPolicies) {
        self.qos = qos;
    }

    /// publish data for matching DataReader
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

    pub fn assert_liveliness(&self) {
        if let LivelinessQosKind::ManualByParticipant = self.qos.liveliness.kind {
            let writer_cmd = WriterCmd::AssertLiveliness;
            self.writer_command_sender
                .send(writer_cmd)
                .expect("couldn't send message");
        }
    }

    /// get DataWriterStatusChanged
    ///
    /// This method is non_blocking, so if failed to get DataReaderStatusChanged, this method returns Err.
    /// DataReader implement mio::Evented, so you can gegister DataReader to mio v0.6's Poll.
    /// Poll DataReader, to ensure get DataWriterStatusChanged.
    pub fn try_recv(&self) -> Result<DataWriterStatusChanged, std::sync::mpsc::TryRecvError> {
        self.writer_state_receiver.try_recv()
    }
}

impl<D: Serialize + DdsData> Evented for DataWriter<D> {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interests: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.writer_state_receiver
            .register(poll, token, interests, opts)
    }
    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interests: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.writer_state_receiver
            .reregister(poll, token, interests, opts)
    }
    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.writer_state_receiver.deregister(poll)
    }
}
