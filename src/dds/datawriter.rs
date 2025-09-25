use crate::dds::{
    key::DdsData,
    publisher::Publisher,
    qos::{policy::LivelinessQosKind, DataWriterQosPolicies},
    topic::Topic,
};
use crate::message::submessage::element::{
    RepresentationIdentifier, SequenceNumber, SerializedPayload, Timestamp,
};
use crate::rtps::{
    cache::{CacheChange, ChangeKind, HistoryCache, InstantHandle},
    writer::*,
};
use crate::structure::GUID;
use alloc::sync::Arc;
use awkernel_sync::rwlock::RwLock;
use core::marker::PhantomData;
use log::{info, warn};
use mio_extras::channel as mio_channel;
use mio_v06::{event::Evented, Poll, PollOpt, Ready, Token};
use serde::Serialize;
use std::io;

/// DDS DataWriter
#[allow(dead_code)]
pub struct DataWriter<D: Serialize + DdsData> {
    data_phantom: PhantomData<D>,
    writer_guid: GUID,
    qos: DataWriterQosPolicies,
    topic: Topic,
    publisher: Publisher,
    whc: Arc<RwLock<HistoryCache>>,
    // last_change_sequence_numberは本来はWriter::new_change()でのCacheChangeの作成時に使用するRTPS Writerのメンバ
    // 本実装ではDataWrtierとRTPS Writerが別スレッドに配置されるため、DataWriterはRTPS Writerのnew_changeを叩けない。
    // そのため、DataWriterがlast_change_sequence_numberを保持している。
    last_change_sequence_number: SequenceNumber,
    // my_guid: GUID, // In RustDDS, DataWriter has guid to drop corresponding RTPSWriter
    // I implement guid for DataWriter when need.
    writer_command_sender: mio_channel::SyncSender<WriterCmd>,
    writer_state_receiver: mio_channel::Receiver<DataWriterStatusChanged>,
}

impl<D: Serialize + DdsData> DataWriter<D> {
    pub(crate) fn new(
        writer_command_sender: mio_channel::SyncSender<WriterCmd>,
        writer_guid: GUID,
        qos: DataWriterQosPolicies,
        topic: Topic,
        publisher: Publisher,
        whc: Arc<RwLock<HistoryCache>>,
        writer_state_receiver: mio_channel::Receiver<DataWriterStatusChanged>,
    ) -> Self {
        Self {
            data_phantom: PhantomData::<D>,
            writer_guid,
            qos,
            topic,
            publisher,
            whc,
            last_change_sequence_number: SequenceNumber(0),
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
    pub fn write(&mut self, data: &D) {
        let ts = Timestamp::now().expect("failed get Timestamp::now()");
        let serialized_payload =
            SerializedPayload::new_from_cdr_data(data, RepresentationIdentifier::CDR_LE);
        self.writer_data_to_hc(ts, serialized_payload);
    }

    pub(crate) fn write_builtin_data(&mut self, data: &D) {
        let ts = Timestamp::now().expect("failed get Timestamp::now()");
        let serialized_payload =
            SerializedPayload::new_from_cdr_data(data, RepresentationIdentifier::PL_CDR_LE);
        self.writer_data_to_hc(ts, serialized_payload);
    }

    fn writer_data_to_hc(&mut self, ts: Timestamp, serialized_payload: SerializedPayload) {
        self.last_change_sequence_number += SequenceNumber(1);
        let a_change = CacheChange::new(
            ChangeKind::Alive,
            self.writer_guid,
            self.last_change_sequence_number,
            ts,
            Some(serialized_payload),
            InstantHandle {},
        );
        if let Err(e) = self.whc.write().add_change(a_change) {
            warn!("DataWriter failed to add change to HistoryCache: {e}");
        } else {
            info!(
                "DataWriter add change to HistoryCache: seq_num: {}\n\tWriter: {}",
                self.last_change_sequence_number.0, self.writer_guid
            );
            self.writer_command_sender
                .send(WriterCmd::WriteData)
                .expect("couldn't send message");
        }
    }

    /// assert liveliness of the DataWriter manually
    ///
    /// DDS 1.4 spec, 2.2.2.4.2.22 assert_liveliness
    /// > This operation need only be used if the LIVELINESS setting is either MANUAL_BY_PARTICIPANT or MANUAL_BY_TOPIC. Otherwise, it has no effect.
    pub fn assert_liveliness(&self) {
        match self.qos.liveliness().kind {
            LivelinessQosKind::Automatic => (),
            LivelinessQosKind::ManualByTopic | LivelinessQosKind::ManualByParticipant => {
                let writer_cmd = WriterCmd::AssertLiveliness;
                self.writer_command_sender
                    .send(writer_cmd)
                    .expect("couldn't send message");
            }
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
