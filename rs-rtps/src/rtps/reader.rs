use crate::rtps::cache::{CacheChange, HistoryCache};
use crate::structure::entity::RTPSEntity;
use crate::structure::guid::GUID;
use std::sync::{Arc, RwLock};

pub struct Reader {
    guid: GUID,
    reader_cache: Arc<RwLock<HistoryCache>>,
}

impl Reader {
    pub fn new(ri: ReaderIngredients) -> Self {
        Self {
            guid: ri.guid,
            reader_cache: ri.rhc,
        }
    }

    pub fn add_change(&mut self, change: CacheChange) {
        self.reader_cache.write().unwrap().add_change(change);
    }
}

pub struct ReaderIngredients {
    pub guid: GUID,
    pub rhc: Arc<RwLock<HistoryCache>>,
}

impl RTPSEntity for Reader {
    fn guid(&self) -> GUID {
        self.guid
    }
}
