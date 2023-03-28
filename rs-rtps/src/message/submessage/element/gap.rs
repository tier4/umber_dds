use crate::message::submessage::{element::*, submessage_flag::GapFlag};
use crate::structure::entityId::*;
use enumflags2::BitFlags;
use speedy::Readable;

#[derive(Readable)]
pub struct Gap {
    readerId: EntityId,
    writerId: EntityId,
    gapStart: SequenceNumber,
    gapList: SequenceNumberSet,
    gapStartGSN: SequenceNumber,
    gapEndGSN: SequenceNumber,
}
