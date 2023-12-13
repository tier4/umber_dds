use crate::message::submessage::element::*;
use crate::structure::entity_id::*;
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct Gap {
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub gap_start: SequenceNumber,
    pub gap_list: SequenceNumberSet,
    pub gap_start_gsn: SequenceNumber,
    pub gap_end_gsn: SequenceNumber,
}
