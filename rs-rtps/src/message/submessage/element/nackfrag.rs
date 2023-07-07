use crate::message::submessage::{element::*, submessage_flag::NackFragFlag};
use crate::structure::entity_id::*;
use enumflags2::BitFlags;
use speedy::Readable;

#[derive(Readable)]
pub struct NackFrag {
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    fragment_number_state: FragmentNumberSet,
    count: Count,
}
