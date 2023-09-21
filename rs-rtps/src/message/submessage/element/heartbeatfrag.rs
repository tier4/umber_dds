use crate::message::submessage::{element::*, submessage_flag::HeartbeatFragFlag};
use crate::structure::entity_id::*;
use enumflags2::BitFlags;
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct HeartbeatFrag {
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    last_fragment_num: FragmentNumber,
    count: Count,
}
