use crate::message::message_header::*;
use crate::structure::{vendorId::*, guid::*};

pub struct InfoSource {
    protocolVersion: ProtocolVersion,
    venderId: VendorId,
    guidPrefix: GuidPrefix,
}
