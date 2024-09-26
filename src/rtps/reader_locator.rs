use crate::message::submessage::element::Locator;
use crate::rtps::cache::CacheChange;

#[derive(PartialEq, Eq)]
pub struct ReaderLocator {
    pub locator: Locator,
    pub requested_changes: Vec<CacheChange>,
    pub unsent_changes: Vec<CacheChange>,
    pub expects_inline_qos: bool,
}

#[allow(dead_code)]
impl ReaderLocator {
    pub fn new(
        locator: Locator,
        unsent_changes: Vec<CacheChange>,
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            locator,
            requested_changes: Vec::new(),
            unsent_changes,
            expects_inline_qos,
        }
    }
    pub fn next_request_change() {}
    pub fn next_unsent_change() {}
    pub fn requested_changes() {}
    pub fn requested_changes_set() {}
    pub fn unsent_changes() {}
}
