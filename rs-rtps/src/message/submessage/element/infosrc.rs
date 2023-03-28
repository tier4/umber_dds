use crate::message::message_header::*;
use crate::structure::{guid::*, vendorId::*};

pub struct InfoSource {
    pub protocolVersion: ProtocolVersion,
    pub vendorId: VendorId,
    pub guidPrefix: GuidPrefix,
}
