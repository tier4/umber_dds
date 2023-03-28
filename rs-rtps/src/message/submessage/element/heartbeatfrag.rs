use crate::message::submessage::{element::*, submessage_flag::HeartbeatFragFlag};
use crate::structure::entityId::*;
use enumflags2::BitFlags;
use speedy::Readable;

#[derive(Readable)]
pub struct HeartbeatFrag {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    lastFragmentNum: FragmentNumber,
    count: Count,
}
