use crate::dds::{qos::DataReaderQosPolicies, subscriber::Subscriber, topic::Topic};
use crate::discovery::structure::cdr::deserialize;
use crate::rtps::{cache::HistoryCache, reader::DataReaderStatusChanged};
use alloc::sync::Arc;
use core::marker::PhantomData;
use mio_extras::channel as mio_channel;
use mio_v06::{event::Evented, Poll, PollOpt, Ready, Token};
use serde::Deserialize;
use std::io;
use std::sync::RwLock;

/// DDS DataReader
pub struct DataReader<D: for<'de> Deserialize<'de>> {
    data_phantom: PhantomData<D>,
    _qos: DataReaderQosPolicies,
    _topic: Topic,
    _subscriber: Subscriber,
    rhc: Arc<RwLock<HistoryCache>>,
    reader_state_receiver: mio_channel::Receiver<DataReaderStatusChanged>,
}

impl<D: for<'de> Deserialize<'de>> DataReader<D> {
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
    pub fn take(&self) -> Vec<D> {
        self.get_data()
    }

    fn get_data(&self) -> Vec<D> {
        let mut hc = self.rhc.write().expect("couldn't read ReaderHistoryCache");
        let changes = hc.get_alive_changes();
        for c in changes.iter() {
            hc.remove_change(c);
        }
        let mut v: Vec<D> = Vec::new();
        for d in changes.iter().filter_map(|c| c.data_value()) {
            match deserialize::<D>(&d.to_bytes()) {
                Ok(neko) => v.push(neko),
                Err(_e) => (),
            }
        }
        v
    }
    fn _remove_changes(&self) {
        let mut hc = self.rhc.write().expect("couldn't write ReaderHistoryCache");
        hc.remove_notalive_changes();
    }
    pub fn get_qos(&self) -> DataReaderQosPolicies {
        self._qos.clone()
    }
    pub fn set_qos(&mut self, qos: DataReaderQosPolicies) {
        self._qos = qos;
    }

    pub fn try_recv(&self) -> Result<DataReaderStatusChanged, std::sync::mpsc::TryRecvError> {
        self.reader_state_receiver.try_recv()
    }
}

impl<D: for<'de> Deserialize<'de>> Evented for DataReader<D> {
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
