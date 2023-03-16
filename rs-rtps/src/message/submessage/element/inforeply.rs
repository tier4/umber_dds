use crate::message::submessage::element::*;

pub struct InfoReply {
    unicastLocaterList: LocatorList,
    multicastLocaterList: LocatorList,
}
