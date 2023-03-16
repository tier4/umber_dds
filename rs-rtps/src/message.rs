pub mod submessage;
pub mod message_receiver;
pub mod message_header;

use crate::{message::{submessage::SubMessage, message_header::*}};

pub struct Message {
    pub header: Header,
    pub submessages: Vec<SubMessage>,
}

impl Message {
    pub fn new(header: Header, submessages: Vec<SubMessage>) -> Message {
        Message { header, submessages }
    }
}
