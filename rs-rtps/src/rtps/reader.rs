use crate::message::submessage::element::Locator;
use crate::policy::ReliabilityQosKind;
use crate::rtps::cache::{CacheChange, HistoryCache};
use crate::structure::{
    duration::Duration, entity::RTPSEntity, guid::GUID, proxy::WriterProxy, topic_kind::TopicKind,
};
use mio_extras::channel as mio_channel;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// RTPS StatefulReader
pub struct Reader {
    // Entity
    guid: GUID,
    // Endpoint
    topic_kind: TopicKind,
    reliability_level: ReliabilityQosKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    // Reader
    expectsinline_qos: bool,
    heartbeat_response_delay: Duration,
    reader_cache: Arc<RwLock<HistoryCache>>,
    // StatefulReader
    writer_proxy: HashMap<GUID, WriterProxy>,
    // This implementation spesific
    reader_ready_notifier: mio_channel::Sender<()>,
}

impl Reader {
    pub fn new(ri: ReaderIngredients) -> Self {
        Self {
            guid: ri.guid,
            topic_kind: ri.topic_kind,
            reliability_level: ri.reliability_level,
            unicast_locator_list: ri.unicast_locator_list,
            multicast_locator_list: ri.multicast_locator_list,
            expectsinline_qos: ri.expectsinline_qos,
            heartbeat_response_delay: ri.heartbeat_response_delay,
            reader_cache: ri.rhc,
            writer_proxy: HashMap::new(),
            reader_ready_notifier: ri.reader_ready_notifier,
        }
    }

    pub fn add_change(&mut self, change: CacheChange) {
        self.reader_cache.write().unwrap().add_change(change);
        self.reader_ready_notifier.send(()).unwrap();
    }
    pub fn matched_writer_add(
        &mut self,
        remote_writer_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: i32,
    ) {
        eprintln!("DataReader::matched_writer_add");
        self.writer_proxy.insert(
            remote_writer_guid,
            WriterProxy::new(
                remote_writer_guid,
                unicast_locator_list,
                multicast_locator_list,
                data_max_size_serialized,
            ),
        );
    }
    pub fn matched_writer_lookup(&self, guid: GUID) -> Option<WriterProxy> {
        unimplemented!("DataReader::matched_writer_lookup");
    }
    pub fn matched_writer_remove(&mut self, proxy: WriterProxy) {
        unimplemented!("DataReader::matched_writer_remove");
    }
}

pub struct ReaderIngredients {
    // Entity
    pub guid: GUID,
    // Endpoint
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityQosKind,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    // Reader
    pub expectsinline_qos: bool,
    pub heartbeat_response_delay: Duration,
    pub rhc: Arc<RwLock<HistoryCache>>,
    // This implementation spesific
    pub reader_ready_notifier: mio_channel::Sender<()>,
}

impl RTPSEntity for Reader {
    fn guid(&self) -> GUID {
        self.guid
    }
}
