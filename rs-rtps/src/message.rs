pub mod message_header;
pub mod message_receiver;
pub mod submessage;

use crate::message::{message_header::*, submessage::SubMessage};

pub struct Message {
    pub header: Header,
    pub submessages: Vec<SubMessage>,
}

impl Message {
    pub fn new(header: Header, submessages: Vec<SubMessage>) -> Message {
        Message {
            header,
            submessages,
        }
    }
}
