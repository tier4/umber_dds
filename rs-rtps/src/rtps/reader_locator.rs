use crate::message::submessage::element::Locator;

pub struct ReaderLocator {
    locator: Locator,
    requested_changes: Vec<()>,
    unsent_changes: Vec<()>,
    expects_inline_qos: bool,
}

impl ReaderLocator {
    pub fn next_request_change() {}
    pub fn next_unsent_change() {}
    pub fn requested_changes() {}
    pub fn requested_changes_set() {}
    pub fn unsent_changes() {}
}
