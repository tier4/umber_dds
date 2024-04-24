use crate::rtps::cache::{CacheChange, HistoryCache};
use crate::structure::{entity::RTPSEntity, guid::GUID, proxy::WriterProxy};
use mio_extras::channel as mio_channel;
use std::sync::{Arc, RwLock};

/// RTPS StatelessReader
pub struct Reader {
    guid: GUID,
    reader_cache: Arc<RwLock<HistoryCache>>,
    reader_ready_notifier: mio_channel::Sender<()>,
}

impl Reader {
    pub fn new(ri: ReaderIngredients) -> Self {
        Self {
            guid: ri.guid,
            reader_cache: ri.rhc,
            reader_ready_notifier: ri.reader_ready_notifier,
        }
    }

    pub fn add_change(&mut self, change: CacheChange) {
        self.reader_cache.write().unwrap().add_change(change);
        self.reader_ready_notifier.send(()).unwrap();
    }
    pub fn matched_writer_add(&mut self, proxy: WriterProxy) {
        unimplemented!("DataReader::matched_writer_add");
    }
    pub fn matched_writer_lookup(&self, guid: GUID) -> Option<WriterProxy> {
        unimplemented!("DataReader::matched_writer_lookup");
    }
    pub fn matched_writer_remove(&mut self, proxy: WriterProxy) {
        unimplemented!("DataReader::matched_writer_remove");
    }
}

pub struct ReaderIngredients {
    pub guid: GUID,
    pub rhc: Arc<RwLock<HistoryCache>>,
    pub reader_ready_notifier: mio_channel::Sender<()>,
}

impl RTPSEntity for Reader {
    fn guid(&self) -> GUID {
        self.guid
    }
}
