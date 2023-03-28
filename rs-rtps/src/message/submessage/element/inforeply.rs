use crate::message::submessage::element::*;
use speedy::Readable;

#[derive(Readable)]
pub struct InfoReply {
    pub unicastLocatorList: LocatorList,
    pub multicastLocatorList: LocatorList,
}
