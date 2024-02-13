use crate::message::message_header::ProtocolVersion;
use crate::message::submessage::element::{Count, Locator, Parameter};
use crate::structure::duration::Duration;
use crate::structure::{
    guid::{GuidPrefix, GUID},
    parameter_id::ParameterId,
    vendor_id::VendorId,
};
use enumflags2::bitflags;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

#[allow(non_camel_case_types)]
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

#[derive(Clone)]
pub struct SPDPdiscoveredParticipantData {
    pub domain_id: u16,
    pub domain_tag: String,
    pub protocol_version: ProtocolVersion,
    pub guid: GUID,
    pub vendor_id: VendorId,
    pub expects_inline_qos: bool,
    pub available_builtin_endpoint: u32, // parameter_id: ParameterId::PID_BUILTIN_ENDPOINT_SET
    pub metarraffic_unicast_locator_list: Vec<Locator>,
    pub metarraffic_multicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub manual_liveliness_count: Count,
    pub lease_duration: Duration,
}

impl SPDPdiscoveredParticipantData {
    pub fn new(
        domain_id: u16,
        domain_tag: String,
        protocol_version: ProtocolVersion,
        guid: GUID,
        vendor_id: VendorId,
        expects_inline_qos: bool,
        available_builtin_endpoint: u32,
        metarraffic_unicast_locator_list: Vec<Locator>,
        metarraffic_multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        manual_liveliness_count: Count,
        lease_duration: Duration,
    ) -> Self {
        Self {
            domain_id,
            domain_tag,
            protocol_version,
            guid,
            vendor_id,
            expects_inline_qos,
            available_builtin_endpoint,
            metarraffic_unicast_locator_list,
            metarraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            manual_liveliness_count,
            lease_duration,
        }
    }
}

impl Serialize for SPDPdiscoveredParticipantData {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("SPDPdiscoveredParticipantData", 4)?;
        // ProtocolVersion
        s.serialize_field("parameterId", &ParameterId::PID_PROTOCOL_VERSION.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("protocol_version: major", &self.protocol_version.major)?;
        s.serialize_field("protocol_version: minor", &self.protocol_version.minor)?;
        s.serialize_field::<u16>("protocol_version: padding", &0)?;

        // VendorId
        s.serialize_field("parameterId", &ParameterId::PID_VENDOR_ID.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("vendorId: major", &self.vendor_id.vendor_id[0])?;
        s.serialize_field("vendorId: minor", &self.vendor_id.vendor_id[1])?;
        s.serialize_field::<u16>("vendorId: padding", &0)?;

        // expects_inline_qos
        s.serialize_field("parameterId", &ParameterId::PID_EXPECTS_INLINE_QOS.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("inline_qos", &self.expects_inline_qos)?;
        s.serialize_field::<u16>("expects_inline_qos: padding", &0)?;

        // participant_guid
        s.serialize_field("parameterId", &ParameterId::PID_PARTICIPANT_GUID.value)?;
        s.serialize_field::<u16>("parameterLength", &16)?;
        s.serialize_field("guid", &self.guid)?;

        // metarraffic_unicast_locator_list
        s.serialize_field(
            "parameterId",
            &ParameterId::PID_METATRAFFIC_UNICAST_LOCATOR.value,
        )?;
        s.serialize_field::<u16>("parameterLength", &24)?;
        // TODO: LocatorListの長さが1以外の場合に対応
        assert_eq!(self.metarraffic_unicast_locator_list.len(), 1);
        s.serialize_field(
            "metarraffic_unicast_locator_list",
            &self.metarraffic_unicast_locator_list[0],
        )?;

        // metarraffic_multicast_locator_list
        s.serialize_field(
            "parameterId",
            &ParameterId::PID_METATRAFFIC_MULTICAST_LOCATOR.value,
        )?;
        s.serialize_field::<u16>("parameterLength", &24)?;
        // TODO: LocatorListの長さが1以外の場合に対応
        assert_eq!(self.metarraffic_multicast_locator_list.len(), 1);
        s.serialize_field(
            "metarraffic_multicast_locator_list",
            &self.metarraffic_multicast_locator_list[0],
        )?;

        // default_unicast_locator_list
        s.serialize_field(
            "parameterId",
            &ParameterId::PID_DEFAULT_UNICAST_LOCATOR.value,
        )?;
        s.serialize_field::<u16>("parameterLength", &24)?;
        // TODO: LocatorListの長さが1以外の場合に対応
        assert_eq!(self.default_unicast_locator_list.len(), 1);
        s.serialize_field(
            "default_unicast_locator_list",
            &self.default_unicast_locator_list[0],
        )?;

        // default_multicast_locator_list
        s.serialize_field(
            "parameterId",
            &ParameterId::PID_DEFAULT_MULTICAST_LOCATOR.value,
        )?;
        s.serialize_field::<u16>("parameterLength", &24)?;
        // TODO: LocatorListの長さが1以外の場合に対応
        assert_eq!(self.default_multicast_locator_list.len(), 1);
        s.serialize_field(
            "default_multicast_locator_list",
            &self.default_multicast_locator_list[0],
        )?;

        // available_builtin_endpoint
        s.serialize_field("parameterId", &ParameterId::PID_BUILTIN_ENDPOINT_SET.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field(
            "available_builtin_endpoint",
            &self.available_builtin_endpoint,
        )?;

        // lease_duration
        s.serialize_field(
            "parameterId",
            &ParameterId::PID_PARTICIPANT_LEASE_DURATION.value,
        )?;
        s.serialize_field::<u16>("parameterLength", &8)?;
        s.serialize_field("lease_duration", &self.lease_duration)?;

        // manual_liveliness_count
        s.serialize_field(
            "parameterId",
            &ParameterId::PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT.value,
        )?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field::<i32>("manual_liveliness_count", &self.manual_liveliness_count)?;

        // sentinel
        s.serialize_field("parameterId", &ParameterId::PID_SENTINEL.value)?;
        s.serialize_field::<u16>("vendorId: padding", &0)?;

        s.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::message::message_header::ProtocolVersion;
    use crate::message::submessage::element::{RepresentationIdentifier, SerializedPayload};

    #[test]
    fn test_serialize() {
        // 現状シリアライズ結果を既存実装のキャプチャと目視で比較しかできない
        // TODO: シリアライズ済のバイナリをソースに埋めて、シリアライズ結果と比較
        let data = SPDPdiscoveredParticipantData::new(
            0,
            "hoge".to_string(),
            ProtocolVersion::PROTOCOLVERSION,
            GUID::new_participant_guid(),
            VendorId::TIER4,
            false,
            0x18000c3f,
            Vec::from([Locator::new_from_ipv4(7410, [192, 168, 208, 3])]),
            Vec::from([Locator::new_from_ipv4(7400, [192, 168, 208, 3])]),
            Vec::from([Locator::new_from_ipv4(7411, [192, 168, 208, 3])]),
            Vec::from([Locator::new_from_ipv4(7400, [192, 168, 208, 3])]),
            0,
            Duration {
                seconds: 20,
                fraction: 0,
            },
        );
        let serialized_payload =
            SerializedPayload::new_from_cdr_data(data, RepresentationIdentifier::PL_CDR_LE);
        let mut serialized = String::new();
        let mut count = 0;
        for b in serialized_payload.value {
            serialized += &format!("{:>02X} ", b);
            count += 1;
            if count % 16 == 0 {
                serialized += "\n";
            } else if count % 8 == 0 {
                serialized += " ";
            }
        }
        println!("{}", serialized);
    }
}
