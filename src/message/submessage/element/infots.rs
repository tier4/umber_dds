use crate::message::submessage::element::*;
use speedy::Readable;

#[derive(Readable)]
pub struct InfoTimestamp {
    pub timestamp: Option<Timestamp>,
}
