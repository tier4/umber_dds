use crate::message::submessage::{element::*, submessage_flag::NackFragFlag};
use crate::structure::entityId::*;
use enumflags2::BitFlags;

pub struct NackFrag {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    fragmentNumberState: FragmentNumberSet,
    count: Count,
}

impl NackFrag {
    pub fn deserialize_data(buffer: &Bytes, flags: BitFlags<NackFragFlag>) {
        todo!();
    }
}
