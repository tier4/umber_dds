use crate::message::submessage::element::*;
use crate::structure::entity_id::*;
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct HeartbeatFrag {
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub last_fragment_num: FragmentNumber,
    pub count: Count,
}
