use enumflags2::bitflags;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[bitflags]
#[repr(u32)]
pub enum BuiltinEndpoint {
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
     */
    DISC_BUILTIN_ENDPOINT_PARTICIPANT_PROXY_ANNOUNCER = 0x01 << 6,
    DISC_BUILTIN_ENDPOINT_PARTICIPANT_PROXY_DETECTOR = 0x01 << 7,
    DISC_BUILTIN_ENDPOINT_PARTICIPANT_STATE_ANNOUNCER = 0x01 << 8,
    DISC_BUILTIN_ENDPOINT_PARTICIPANT_STATE_DETECTOR = 0x01 << 9,

    BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER = 0x01 << 10,
    BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER = 0x01 << 11,
    /*
     * Bits 12-15 have been reserved by the DDS-Xtypes 1.2 Specification
     * and future revisions thereof.
     */
    TypeLookupServiceRequestDataWriter = 0x01 << 12,
    TypeLookupServiceRequestDataReader = 0x01 << 13,
    TypeLookupServiceReplyDataWriter = 0x01 << 14,
    TypeLookupServiceReplyDataReader = 0x01 << 15,

    /*
     * Bits 16-27 have been reserved by the DDS-Security 1.1 Specification
     * and future revisions thereof.
     */
    SEDPbuiltinPublicationsSecureWriter = 0x01 << 16,
    SEDPbuiltinPublicationsSecureReader = 0x01 << 17,
    SEDPbuiltinSubscriptionsSecureWriter = 0x01 << 18,
    SEDPbuiltinSubscriptionsSecureReader = 0x01 << 19,
    BuiltinParticipantMessageSecureWriter = 0x01 << 20,
    BuiltinParticipantMessageSecureReader = 0x01 << 21,
    BuiltinParticipantStatelessMessageWriter = 0x01 << 22,
    BuiltinParticipantStatelessMessageReader = 0x01 << 23,
    BuiltinParticipantVolatileMessageSecureWriter = 0x01 << 24,
    BuiltinParticipantVolatileMessageSecureReader = 0x01 << 25,
    SPDPbuiltinParticipantSecureWriter = 0x01 << 26,
    SPDPbuiltinParticipantSecureReader = 0x01 << 27,

    DISC_BUILTIN_ENDPOINT_TOPICS_ANNOUNCER = 0x01 << 28,
    DISC_BUILTIN_ENDPOINT_TOPICS_DETECTOR = 0x01 << 29,
}
