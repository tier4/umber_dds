use crate::message::message_header::*;
use crate::structure::{guid::*, vendor_id::*};
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct InfoSource {
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}
