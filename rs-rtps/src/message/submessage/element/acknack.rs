use crate::structure::entityId::*;
use crate::message::submessage::element::*;

pub struct AckNack {
    readerId: EntityId,
    writerId: EntityId,
    readerSNState: SequenceNumberSet,
    count: Count,
}
