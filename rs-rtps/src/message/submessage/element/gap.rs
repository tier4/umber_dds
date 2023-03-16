use crate::structure::entityId::*;
use crate::message::submessage::element::*;

struct Gap {
    readerId: EntityId,
    writerId: EntityId,
    gapStart: SequenceNumber,
    gapList: SequenceNumberSet,
    gapStartGSN: SequenceNumber,
    gapEndGSN: SequenceNumber,
}
