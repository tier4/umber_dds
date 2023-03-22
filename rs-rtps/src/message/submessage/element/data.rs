use crate::message::submessage::{element::*, submessage_flag::DataFlag};
use crate::structure::entityId::*;
use enumflags2::BitFlags;

pub struct Data {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    inlineQos: ParameterList,
    serializedPayload: SerializedPayload,
}

impl Data {
    pub fn deserialize_data(buffer: &Bytes, flags: BitFlags<DataFlag>) -> Self {
        todo!();
    }
}
