use speedy::{Readable, Writable};

// from jhelovuo/RustDDS(https://github.com/jhelovuo/RustDDS.git), src/structure/parameter_id.rs

#[derive(Readable, Writable, PartialEq, Eq)]
pub struct ParameterId {
    pub value: u16,
}

impl ParameterId {
    #![allow(dead_code)] // since we do not necessarily use all of the named constants, but that's ok
    pub const PID_PAD: Self = Self { value: 0x0000 };
    pub const PID_SENTINEL: Self = Self { value: 0x0001 };
    pub const PID_USER_DATA: Self = Self { value: 0x002c };
    pub const PID_TOPIC_NAME: Self = Self { value: 0x0005 };
    pub const PID_TYPE_NAME: Self = Self { value: 0x0007 };
    pub const PID_DOMAIN_ID: Self = Self { value: 0x000f };
    pub const PID_GROUP_DATA: Self = Self { value: 0x002d };
    pub const PID_TOPIC_DATA: Self = Self { value: 0x002e };
    pub const PID_DURABILITY: Self = Self { value: 0x001d };
    pub const PID_DURABILITY_SERVICE: Self = Self { value: 0x001e };
    pub const PID_DEADLINE: Self = Self { value: 0x0023 };
    pub const PID_LATENCY_BUDGET: Self = Self { value: 0x0027 };
    pub const PID_LIVELINESS: Self = Self { value: 0x001b };
    pub const PID_RELIABILITY: Self = Self { value: 0x001a };
    pub const PID_LIFESPAN: Self = Self { value: 0x002b };
    pub const PID_DESTINATION_ORDER: Self = Self { value: 0x0025 };
    pub const PID_HISTORY: Self = Self { value: 0x0040 };
    pub const PID_RESOURCE_LIMITS: Self = Self { value: 0x0041 };
    pub const PID_OWNERSHIP: Self = Self { value: 0x001f };
    pub const PID_OWNERSHIP_STRENGTH: Self = Self { value: 0x0006 };
    pub const PID_PRESENTATION: Self = Self { value: 0x0021 };
    pub const PID_PARTITION: Self = Self { value: 0x0029 };
    pub const PID_TIME_BASED_FILTER: Self = Self { value: 0x0004 };
    pub const PID_TRANSPORT_PRIO: Self = Self { value: 0x0049 };
    pub const PID_PROTOCOL_VERSION: Self = Self { value: 0x0015 };
    pub const PID_VENDOR_ID: Self = Self { value: 0x0016 };
    pub const PID_UNICAST_LOCATOR: Self = Self { value: 0x002f };
    pub const PID_MULTICAST_LOCATOR: Self = Self { value: 0x0030 };
    pub const PID_MULTICAST_IPADDRESS: Self = Self { value: 0x0011 };
    pub const PID_DEFAULT_UNICAST_LOCATOR: Self = Self { value: 0x0031 };
    pub const PID_DEFAULT_MULTICAST_LOCATOR: Self = Self { value: 0x0048 };
    pub const PID_METATRAFFIC_UNICAST_LOCATOR: Self = Self { value: 0x0032 };
    pub const PID_METATRAFFIC_MULTICAST_LOCATOR: Self = Self { value: 0x0033 };
    pub const PID_DEFAULT_UNICAST_IPADDRESS: Self = Self { value: 0x000c };
    pub const PID_DEFAULT_UNICAST_PORT: Self = Self { value: 0x000e };
    pub const PID_METATRAFFIC_UNICAST_IPADDRESS: Self = Self { value: 0x0045 };
    pub const PID_METATRAFFIC_UNICAST_PORT: Self = Self { value: 0x000d };
    pub const PID_METATRAFFIC_MULTICAST_IPADDRESS: Self = Self { value: 0x000b };
    pub const PID_METATRAFFIC_MULTICAST_PORT: Self = Self { value: 0x0046 };
    pub const PID_EXPECTS_INLINE_QOS: Self = Self { value: 0x0043 };
    pub const PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT: Self = Self { value: 0x0034 };
    pub const PID_PARTICIPANT_BUILTIN_ENDPOINTS: Self = Self { value: 0x0044 };
    pub const PID_PARTICIPANT_LEASE_DURATION: Self = Self { value: 0x0002 };
    pub const PID_CONTENT_FILTER_PROPERTY: Self = Self { value: 0x0035 };
    pub const PID_PARTICIPANT_GUID: Self = Self { value: 0x0050 };
    pub const PID_GROUP_GUID: Self = Self { value: 0x0052 };
    pub const PID_GROUP_ENTITYID: Self = Self { value: 0x0053 };
    pub const PID_BUILTIN_ENDPOINT_SET: Self = Self { value: 0x0058 };
    pub const PID_ENDPOINT_GUID: Self = Self { value: 0x005a };
    pub const PID_BUILTIN_ENDPOINT_QOS: Self = Self { value: 0x0077 };
    pub const PID_PROPERTY_LIST: Self = Self { value: 0x0059 };
    pub const PID_TYPE_MAX_SIZE_SERIALIZED: Self = Self { value: 0x0060 };
    pub const PID_ENTITY_NAME: Self = Self { value: 0x0062 };
    pub const PID_KEY_HASH: Self = Self { value: 0x0070 };
    pub const PID_STATUS_INFO: Self = Self { value: 0x0071 };
    pub const PID_DOMAIN_TAG: Self = Self { value: 0x4014 };

    // From Specification "Remote Procedure Calls over DDS v1.0"
    // Section 7.6.2.1.1 Extended PublicationBuiltin TopicData and
    // 7.6.2.1.2 Extended SubscriptionBuiltinTopicData
    pub const PID_SERVICE_INSTANCE_NAME: Self = Self { value: 0x0080 };
    pub const PID_RELATED_ENTITY_GUID: Self = Self { value: 0x0081 };
    pub const PID_TOPIC_ALIASES: Self = Self { value: 0x0082 };
    // Section "7.8.2 Request and Reply Correlation in the Enhanced Service
    // Profile": ...a new parameter id PID_RELATED_SAMPLE_IDENTITY with value
    // 0x0083
    //
    // But then again, the actual PID on the wire seems to be 0x800f, at least in
    // eProsima FastRTPS and RTI Connext. eProsima sources even have the value
    // 0x0083 commented out.
    // Wireshark calls this "PID_RELATED_ORIGINAL_WRITER_INFO".
    pub const PID_RELATED_SAMPLE_IDENTITY: Self = Self { value: /*0x0083*/ 0x800f };
}
