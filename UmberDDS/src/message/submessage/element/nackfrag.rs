use crate::message::submessage::element::*;
use crate::structure::entity_id::*;
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct NackFrag {
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub fragment_number_state: FragmentNumberSet,
    pub count: Count,
}
