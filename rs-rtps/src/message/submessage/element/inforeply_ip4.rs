use crate::message::submessage::element::*;
use speedy::Readable;

#[derive(Readable)]
pub struct InfoReplyIp4 {
    pub unicast_locator_list: LocatorList,
    pub multicast_locator_list: Option<LocatorList>,
}