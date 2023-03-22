use crate::message::submessage::{element::*, submessage_flag::GapFlag};
use crate::structure::entityId::*;
use enumflags2::BitFlags;

pub struct Gap {
    readerId: EntityId,
    writerId: EntityId,
    gapStart: SequenceNumber,
    gapList: SequenceNumberSet,
    gapStartGSN: SequenceNumber,
    gapEndGSN: SequenceNumber,
}

impl Gap {
    pub fn deserialize_data(buffer: &Bytes, flags: BitFlags<GapFlag>) {
        todo!();
    }
}
