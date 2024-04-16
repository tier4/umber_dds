use crate::dds::{qos::QosPolicies, subscriber::Subscriber, topic::Topic};
use crate::discovery::structure::cdr::deserialize;
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

    pub fn take(&self) -> Vec<D> {
        while let Ok(_) = self.reader_ready_receiver.try_recv() {}
        let d = self.get_change();
        self.remove_changes();
        d
    }
    fn get_change(&self) -> Vec<D> {
        let hc = self.rhc.read().unwrap();
        let data = hc.get_changes();
        let mut v: Vec<D> = Vec::new();
        for d in data {
            match d {
                Some(cd) => match deserialize::<D>(&cd.data()) {
                    Ok(neko) => v.push(neko),
                    Err(_e) => (),
                },
                None => (),
            }
        }
        v
    }
    fn remove_changes(&self) {
        let mut hc = self.rhc.write().unwrap();
        hc.remove_changes();
    }
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
