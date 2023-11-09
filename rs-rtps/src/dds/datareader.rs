use crate::dds::{qos::QosPolicies, subscriber::Subscriber, topic::Topic};
use crate::rtps::cache::HistoryCache;
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

pub struct DataReader<D: Serialize> {
    data_phantom: PhantomData<D>,
    qos: QosPolicies,
    topic: Topic,
    subscriber: Subscriber,
    rhc: Arc<RwLock<HistoryCache>>,
}

impl<D: Serialize> DataReader<D> {
    pub fn new(
        qos: QosPolicies,
        topic: Topic,
        subscriber: Subscriber,
        rhc: Arc<RwLock<HistoryCache>>,
    ) -> Self {
        DataReader {
            data_phantom: PhantomData::<D>,
            qos,
            topic,
            subscriber,
            rhc,
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
