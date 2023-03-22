use crate::message::message_header::*;
use crate::structure::{guid::*, vendorId::*};

pub struct InfoSource {
    protocolVersion: ProtocolVersion,
    venderId: VendorId,
    guidPrefix: GuidPrefix,
}
