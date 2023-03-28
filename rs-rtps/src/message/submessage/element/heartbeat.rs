use crate::message::submessage::{element::*, submessage_flag::HeartbeatFlag};
use crate::structure::entityId::*;
use enumflags2::BitFlags;
use speedy::Readable;

#[derive(Readable)]
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
