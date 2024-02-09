use crate::dds::qos::QosPolicies;
use crate::dds::topic::Topic;
use crate::structure::guid::GUID;
use crate::structure::topic_kind::TopicKind;
use bytes::Bytes;

struct ParticipantMessageData {
    guid: GUID,
    kind: [u8; 4],
    data: Bytes,
}
