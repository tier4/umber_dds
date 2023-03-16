use crate::structure::entityId::*;
use crate::message::submessage::element::*;

pub struct Heartbeat {
    readerId: EntityId,
    writerId: EntityId,
    firstSN: SequenceNumber,
    lastSN: SequenceNumber,
    count: Count,
    currentGSN: SequenceNumber,
    firstGSN: SequenceNumber,
    lastGSN: SequenceNumber,
    writerSet: GroupDigest,
    secureWriterSet: GroupDigest,
}
