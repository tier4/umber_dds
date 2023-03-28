use crate::message::message_header::*;
use crate::structure::{guid::*, vendorId::*};
use speedy::Readable;

#[derive(Readable)]
pub struct InfoSource {
    pub protocolVersion: ProtocolVersion,
    pub vendorId: VendorId,
    pub guidPrefix: GuidPrefix,
}
