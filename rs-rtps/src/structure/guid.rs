use crate::structure::entityId::*;
use rand;
use speedy::Readable;

#[derive(Clone, Copy)]
pub struct GUID {
    pub guidPrefix: GuidPrefix,
    pub entityId: EntityId,
}

impl GUID {
    pub const UNKNOW: Self = Self {
        guidPrefix: GuidPrefix::UNKNOW,
        entityId: EntityId::UNKNOW,
    };

    pub fn new(guidPrefix: GuidPrefix, entityId: EntityId) -> Self {
        Self {
            guidPrefix,
            entityId,
        }
    }

    pub fn new_from_id(&self, entityId: EntityId) -> Self {
        Self {
            guidPrefix: self.guidPrefix,
            entityId,
        }
    }

    pub fn new_participant_guid() -> Self {
        Self {
            guidPrefix: GuidPrefix::new(),
            entityId: EntityId::PARTICIPANT,
        }
    }
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
        // spec 8.2.4.2 The GUIDs of RTPS Participants
        // The implementation is free to chose the prefix,
        // as long as every Participant in the Domain has a unique GUID.
        bytes[0] = crate::structure::vendorId::VendorId::THIS_IMPLEMENTATION.vendor_id[0];
        bytes[1] = crate::structure::vendorId::VendorId::THIS_IMPLEMENTATION.vendor_id[1];
        Self { guid_prefix: bytes }
    }
}
