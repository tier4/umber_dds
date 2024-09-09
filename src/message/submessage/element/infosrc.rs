use crate::message::message_header::*;
use crate::structure::{GuidPrefix, VendorId};
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct InfoSource {
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}
