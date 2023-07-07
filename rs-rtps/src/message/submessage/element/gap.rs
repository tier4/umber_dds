use crate::message::submessage::{element::*, submessage_flag::GapFlag};
use crate::structure::entity_id::*;
use enumflags2::BitFlags;
use speedy::Readable;

#[derive(Readable)]
pub struct Gap {
    reader_id: EntityId,
    writer_id: EntityId,
    gap_start: SequenceNumber,
    gap_list: SequenceNumberSet,
    gap_start_gsn: SequenceNumber,
    gap_end_gsn: SequenceNumber,
}
