use crate::structure::entity_id::*;
use rand::{self, rngs::SmallRng, Rng};
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
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

    pub fn new_participant_guid(small_rng: &mut SmallRng) -> Self {
        Self {
            guid_prefix: GuidPrefix::new(small_rng),
            entity_id: EntityId::PARTICIPANT,
        }
    }
}

#[derive(
    Readable, Writable, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct GuidPrefix {
    pub guid_prefix: [u8; 12],
}
impl GuidPrefix {
    pub const UNKNOW: Self = Self {
        guid_prefix: [0x00; 12],
    };

    // rtps 2.3 sprc, 9.3.1.5
    // To comply with this specification, implementations of the RTPS protocol shall set the first two bytes
    // of the guidPrefix to match their assigned vendorId (see 8.3.3.1.3). This ensures that the guidPrefix
    // remains unique within a DDS Domain even if multiple implementations of the protocol are used.
    // guid_prefix[0] = venderId[0]
    // guid_prefix[1] = venderId[1]
    pub fn new(small_rng: &mut SmallRng) -> Self {
        let mut bytes: [u8; 12] = small_rng.gen();

        // spec 8.2.4.2 The GUIDs of RTPS Participants
        // The implementation is free to chose the prefix,
        // as long as every Participant in the Domain has a unique GUID.
        //
        // This implementation chose using random number to GuidPrefix.
        bytes[0] = crate::structure::vendor_id::VendorId::THIS_IMPLEMENTATION.vendor_id[0];
        bytes[1] = crate::structure::vendor_id::VendorId::THIS_IMPLEMENTATION.vendor_id[1];
        Self { guid_prefix: bytes }
    }
}
