use crate::structure::entityId::*;
use rand;
use speedy::Readable;

pub struct GUID {
    guidPrefix: GuidPrefix,
    entityId: EntityId,
}

impl GUID {
    pub const UNKNOW: Self = Self {
        guidPrefix: GuidPrefix::UNKNOW,
        entityId: EntityId::UNKNOW,
    };
}

#[derive(Readable, Debug, Clone, Copy, PartialEq)]
pub struct GuidPrefix {
    pub guid_prefix: [u8; 12],
}
impl GuidPrefix {
    pub const UNKNOW: Self = Self {
        guid_prefix: [0x00; 12],
    };

    // sprc 9.3.1.5
    // guidPrefix[0] = venderId[0]
    // guidPrefix[1] = venderId[1]
    pub fn new() -> Self {
        let mut bytes: [u8; 12] = rand::random();
        bytes[0] = crate::structure::vendorId::VendorId::THIS_IMPLEMENTATION.vendor_id[0];
        bytes[1] = crate::structure::vendorId::VendorId::THIS_IMPLEMENTATION.vendor_id[1];
        Self { guid_prefix: bytes }
    }
}
