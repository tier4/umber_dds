use crate::message::message_header::ProtocolVersion;
use crate::message::submessage::element::{Count, Locator, Parameter};
use crate::structure::{guid::GuidPrefix, vendor_id::VendorId};
use enumflags2::{bitflags, BitFlags};

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u32)]
pub enum BuiltinEndpointSet {
    DISC_BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER = 0x01 << 0,
    DISC_BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR = 0x01 << 1,
    DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER = 0x01 << 2,
    DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR = 0x01 << 3,
    DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER = 0x01 << 4,
    DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR = 0x01 << 5,
    /*
     * RTPS spec 2.3
     * 9.3.2 Mapping of the Types that Appear Within Submessages or Built-in Topic Data
     * The following have been deprecated in version 2.4 of the specification.
     * These bits should not be used by versions of the protocol equal to or
     * newer than the deprecated version unless they are used with the same meaning
     * as in versions prior to the deprecated version.
    DISC_BUILTIN_ENDPOINT_PARTICIPANT_PROXY_ANNOUNCER = 0x01 << 6,
    DISC_BUILTIN_ENDPOINT_PARTICIPANT_PROXY_DETECTOR = 0x01 << 7,
    DISC_BUILTIN_ENDPOINT_PARTICIPANT_STATE_ANNOUNCER = 0x01 << 8,
    DISC_BUILTIN_ENDPOINT_PARTICIPANT_STATE_DETECTOR = 0x01 << 9,
    */
    BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER = 0x01 << 10,
    BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER = 0x01 << 11,
    /*
     * Bits 12-15 have been reserved by the DDS-Xtypes 1.2 Specification
     * and future revisions thereof.
     * Bits 16-27 have been reserved by the DDS-Security 1.1 Specification
     * and future revisions thereof.
     */
    DISC_BUILTIN_ENDPOINT_TOPICS_ANNOUNCER = 0x01 << 28,
    DISC_BUILTIN_ENDPOINT_TOPICS_DETECTOR = 0x01 << 29,
}

pub struct ParticipantProxy {
    pub domain_id: u16,
    pub domain_tag: String,
    pub protocol_version: ProtocolVersion,
    pub guid_prefix: GuidPrefix,
    pub vendor_id: VendorId,
    pub expects_inline_qos: bool,
    pub available_builtin_endpoint: Parameter, // parameter_id: ParameterId::PID_BUILTIN_ENDPOINT_SET
    pub metarraffic_unicast_locator_list: Vec<Locator>,
    pub metarraffic_multicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub manual_liveliness_count: Count,
}
