use crate::message::submessage::{element::*, submessage_flag::AckNackFlag};
use crate::structure::entity_id::*;
use enumflags2::BitFlags;
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct AckNack {
    reader_id: EntityId,
    writer_id: EntityId,
    reader_sn_state: SequenceNumberSet,
    count: Count,
}
