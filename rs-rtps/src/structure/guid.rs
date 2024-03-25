use crate::structure::entity_id::*;
use rand;
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GUID {
    pub guid_prefix: GuidPrefix,
    pub entity_id: EntityId,
}

impl GUID {
    pub const UNKNOW: Self = Self {
        guid_prefix: GuidPrefix::UNKNOW,
        entity_id: EntityId::UNKNOW,
    };

    pub fn new(guid_prefix: GuidPrefix, entity_id: EntityId) -> Self {
        Self {
            guid_prefix,
            entity_id,
        }
    }

    pub fn new_participant_guid() -> Self {
        Self {
            guid_prefix: GuidPrefix::new(),
            entity_id: EntityId::PARTICIPANT,
        }
    }
}

#[derive(Readable, Writable, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuidPrefix {
    pub guid_prefix: [u8; 12],
}
impl GuidPrefix {
    pub const UNKNOW: Self = Self {
        guid_prefix: [0x00; 12],
    };

    // sprc 9.3.1.5
    // guid_prefix[0] = venderId[0]
    // guid_prefix[1] = venderId[1]
    pub fn new() -> Self {
        let mut bytes: [u8; 12] = rand::random();
        // spec 8.2.4.2 The GUIDs of RTPS Participants
        // The implementation is free to chose the prefix,
        // as long as every Participant in the Domain has a unique GUID.
        bytes[0] = crate::structure::vendor_id::VendorId::THIS_IMPLEMENTATION.vendor_id[0];
        bytes[1] = crate::structure::vendor_id::VendorId::THIS_IMPLEMENTATION.vendor_id[1];
        Self { guid_prefix: bytes }
    }
}
