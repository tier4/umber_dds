use crate::message::submessage::{element::*, submessage_flag::HeartbeatFlag};
use crate::structure::entityId::*;
use enumflags2::BitFlags;

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

impl Heartbeat {
    pub fn deserialize_data(buffer: &Bytes, flags: BitFlags<HeartbeatFlag>) {
        todo!();
    }
}
