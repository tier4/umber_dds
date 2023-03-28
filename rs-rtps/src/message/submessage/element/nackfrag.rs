use crate::message::submessage::{element::*, submessage_flag::NackFragFlag};
use crate::structure::entityId::*;
use enumflags2::BitFlags;
use speedy::Readable;

#[derive(Readable)]
pub struct NackFrag {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    fragmentNumberState: FragmentNumberSet,
    count: Count,
}
