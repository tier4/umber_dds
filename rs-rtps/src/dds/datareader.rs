use crate::dds::{qos::QosPolicies, subscriber::Subscriber, topic::Topic};
use crate::rtps::cache::HistoryCache;
use mio_extras::channel as mio_channel;
use mio_v06::{event::Evented, Poll, PollOpt, Ready, Token};
use serde::Deserialize;
use std::io;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

pub struct DataReader<D: for<'de> Deserialize<'de>> {
    data_phantom: PhantomData<D>,
    qos: QosPolicies,
    topic: Topic,
    subscriber: Subscriber,
    rhc: Arc<RwLock<HistoryCache>>,
    reader_ready_receiver: mio_channel::Receiver<()>,
}

impl<D: for<'de> Deserialize<'de>> DataReader<D> {
    pub fn new(
        qos: QosPolicies,
        topic: Topic,
        subscriber: Subscriber,
        rhc: Arc<RwLock<HistoryCache>>,
        reader_ready_receiver: mio_channel::Receiver<()>,
    ) -> Self {
        DataReader {
            data_phantom: PhantomData::<D>,
            qos,
            topic,
            subscriber,
            rhc,
            reader_ready_receiver,
        }
    }
    pub fn take(&self) -> Option<D> {
        match self.get_change() {
            Some(d) => {
                self.remove_change();
                Some(d)
            }
            None => None,
        }
    }
    fn get_change(&self) -> Option<D> {
        todo!();
        // get change from reader HistoryCache
    }
    fn remove_change(&self) {}
    pub fn get_qos() {}
    pub fn set_qos() {}
}

impl<D: for<'de> Deserialize<'de>> Evented for DataReader<D> {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interests: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.reader_ready_receiver
            .register(poll, token, interests, opts)
    }
    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interests: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.reader_ready_receiver
            .reregister(poll, token, interests, opts)
    }
    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.reader_ready_receiver.deregister(poll)
    }
}
