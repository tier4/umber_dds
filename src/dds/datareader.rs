use crate::dds::{qos::DataReaderQosPolicies, subscriber::Subscriber, topic::Topic};
use crate::discovery::structure::cdr::deserialize;
use crate::rtps::{cache::HistoryCache, reader::DataReaderStatusChanged};
use crate::DdsData;
use alloc::sync::Arc;
use awkernel_sync::rwlock::RwLock;
use core::marker::PhantomData;
use mio_extras::channel as mio_channel;
use mio_v06::{event::Evented, Poll, PollOpt, Ready, Token};
use serde::Deserialize;
use std::io;

/// DDS DataReader
pub struct DataReader<D: for<'de> Deserialize<'de> + DdsData> {
    data_phantom: PhantomData<D>,
    _qos: DataReaderQosPolicies,
    _topic: Topic,
    _subscriber: Subscriber,
    rhc: Arc<RwLock<HistoryCache>>,
    reader_state_receiver: mio_channel::Receiver<DataReaderStatusChanged>,
}

impl<D: for<'de> Deserialize<'de> + DdsData> DataReader<D> {
    pub(crate) fn new(
        qos: DataReaderQosPolicies,
        topic: Topic,
        subscriber: Subscriber,
        rhc: Arc<RwLock<HistoryCache>>,
        reader_state_receiver: mio_channel::Receiver<DataReaderStatusChanged>,
    ) -> Self {
        DataReader {
            data_phantom: PhantomData::<D>,
            _qos: qos,
            _topic: topic,
            _subscriber: subscriber,
            rhc,
            reader_state_receiver,
        }
    }

    /// get available data received from DataWriter
    ///
    /// this function may return empty Vec.
    /// DataReader implement mio::Evented, so you can gegister DataReader to mio v0.6's Poll.
    /// poll DataReader, to ensure taking data.
    ///
    /// The (i+1)-th element of the return value of this method is newer than the i-th element.
    ///
    /// When History QoS is set to KeepLast: depth N, this method returns an Vec with a maximum length of N elements.
    pub fn take(&self) -> Vec<D> {
        self.get_data()
    }

    fn get_data(&self) -> Vec<D> {
        let mut hc = self.rhc.write();
        let (keys, changes) = hc.get_ready_changes();
        let mut v: Vec<D> = Vec::new();
        for d in changes.iter().filter_map(|change| change.data_value()) {
            match deserialize::<D>(&d.to_bytes()) {
                Ok(neko) => v.push(neko),
                Err(_e) => (),
            }
        }
        for key in keys.iter() {
            hc.remove_change(key);
        }
        v
    }
    pub fn get_qos(&self) -> DataReaderQosPolicies {
        self._qos.clone()
    }
    pub fn set_qos(&mut self, qos: DataReaderQosPolicies) {
        self._qos = qos;
    }

    /// get DataReaderStatusChanged
    ///
    /// This method is non_blocking, so if failed to get DataReaderStatusChanged, this method returns Err.
    /// DataReader implement mio::Evented, so you can gegister DataReader to mio v0.6's Poll.
    /// Poll DataReader, to ensure get DataReaderStatusChanged.
    pub fn try_recv(&self) -> Result<DataReaderStatusChanged, std::sync::mpsc::TryRecvError> {
        self.reader_state_receiver.try_recv()
    }
}

impl<D: for<'de> Deserialize<'de> + DdsData> Evented for DataReader<D> {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interests: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.reader_state_receiver
            .register(poll, token, interests, opts)
    }
    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interests: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.reader_state_receiver
            .reregister(poll, token, interests, opts)
    }
    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.reader_state_receiver.deregister(poll)
    }
}
