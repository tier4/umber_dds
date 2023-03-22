use crate::structure::entityId::*;
use crate::message::submessage::{element::*, submessage_flag::AckNackFlag};
use enumflags2::BitFlags;

pub struct AckNack {
    readerId: EntityId,
    writerId: EntityId,
    readerSNState: SequenceNumberSet,
    count: Count,
}

impl AckNack {
    pub fn deserialize_data(buffer: &Bytes, flags: BitFlags<AckNackFlag>) {
        todo!();
    }
}
