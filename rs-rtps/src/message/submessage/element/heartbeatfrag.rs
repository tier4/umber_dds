use crate::message::submessage::{element::*, submessage_flag::HeartbeatFragFlag};
use crate::structure::entity_id::*;
use enumflags2::BitFlags;
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct HeartbeatFrag {
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub last_fragment_num: FragmentNumber,
    pub count: Count,
}
