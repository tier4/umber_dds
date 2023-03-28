use crate::message::submessage::element::*;

pub struct InfoReply {
    pub unicastLocatorList: LocatorList,
    pub multicastLocatorList: LocatorList,
}
