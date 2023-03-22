use crate::structure::entityId::*;
use crate::message::submessage::{element::*, submessage_flag::HeartbeatFragFlag};
use enumflags2::BitFlags;

pub struct HeartbeatFrag {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    lastFragmentNum: FragmentNumber,
    count: Count,
}

impl HeartbeatFragFlag {
    pub fn deserialize_data(buffer: &Bytes, flags: BitFlags<HeartbeatFragFlag>) {
        todo!();
    }
}
