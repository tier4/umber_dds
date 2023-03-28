use crate::message::submessage::{element::*, submessage_flag::AckNackFlag};
use crate::structure::entityId::*;
use enumflags2::BitFlags;
use speedy::Readable;

#[derive(Readable)]
pub struct AckNack {
    readerId: EntityId,
    writerId: EntityId,
    readerSNState: SequenceNumberSet,
    count: Count,
}
