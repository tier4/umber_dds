use crate::structure::entityId::*;
use crate::message::submessage::element::*;

pub struct NackFrag {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    fragmentNumberState: FragmentNumberSet,
    count: Count,
}
