use crate::structure::entityId::*;
use crate::message::submessage::element::*;

pub struct HeartbeatFrag {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    lastFragmentNum: FragmentNumber,
    count: Count,
}
