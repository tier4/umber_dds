use crate::message::submessage::element::*;
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct InfoReplyIp4 {
    pub unicast_locator_list: LocatorList,
    pub multicast_locator_list: Option<LocatorList>,
}
