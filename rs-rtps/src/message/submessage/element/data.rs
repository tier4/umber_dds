use crate::structure::entityId::*;
use crate::message::submessage::element::*;

struct Data {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    inlineQos: ParameterList,
    serializedPayload: SerializedPayload,
}
