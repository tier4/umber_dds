use crate::message::submessage::{element::*, submessage_flag::HeartbeatFlag};
use crate::structure::entity_id::*;
use enumflags2::BitFlags;
use speedy::Readable;

#[derive(Readable)]
pub struct Heartbeat {
    reader_id: EntityId,
    writer_id: EntityId,
    first_sn: SequenceNumber,
    last_sn: SequenceNumber,
    count: Count,
    current_gsn: SequenceNumber,
    first_gsn: SequenceNumber,
    last_gsn: SequenceNumber,
    writer_set: GroupDigest,
    secure_writer_set: GroupDigest,
}
