use crate::message::submessage::{element::*, submessage_flag::DataFragFlag};
use crate::structure::entityId::*;
use enumflags2::BitFlags;

pub struct DataFrag {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    fragmentStaringNum: FragmentNumber,
    fragmentInSubmessage: u16,
    inlineQos: ParameterList,
    serializedPayload: SerializedPayload,
}

impl DataFrag {
    pub fn deserialize(buffer: &Bytes, flags: BitFlags<DataFragFlag>) -> std::io::Result<Self> {
        todo!(); // TODO:
    }
}
