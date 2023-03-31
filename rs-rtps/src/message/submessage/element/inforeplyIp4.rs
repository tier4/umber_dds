use crate::message::submessage::element::*;
use speedy::Readable;

#[derive(Readable)]
pub struct InfoReplyIp4 {
    pub unicastLocatorList: LocatorList,
    pub multicastLocatorList: Option<LocatorList>,
}
