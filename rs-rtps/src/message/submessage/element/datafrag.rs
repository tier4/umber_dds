use crate::structure::entityId::*;
use crate::message::submessage::element::*;

pub struct DataFrag {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    fragmentStaringNum: FragmentNumber,
    fragmentInSubmessage: u16,
    inlineQos: ParameterList,
    serializedPayload: SerializedPayload,
}
