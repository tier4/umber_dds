use crate::message::message_header::ProtocolVersion;
use crate::message::submessage::element::{Count, Locator, Parameter};
use crate::structure::duration::Duration;
use crate::structure::{
    guid::{GuidPrefix, GUID},
    parameter_id::ParameterId,
    vendor_id::VendorId,
};
use enumflags2::bitflags;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::fmt;

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

#[derive(Clone, Default)]
pub struct SDPBuiltinData {
    pub domain_id: Option<u16>,
    pub domain_tag: Option<String>,
    pub protocol_version: Option<ProtocolVersion>,
    pub guid: Option<GUID>,
    pub vendor_id: Option<VendorId>,
    pub expects_inline_qos: Option<bool>,
    pub available_builtin_endpoint: Option<u32>, // parameter_id: ParameterId::PID_BUILTIN_ENDPOINT_SET
    pub metarraffic_unicast_locator_list: Option<Vec<Locator>>,
    pub metarraffic_multicast_locator_list: Option<Vec<Locator>>,
    pub default_multicast_locator_list: Option<Vec<Locator>>,
    pub default_unicast_locator_list: Option<Vec<Locator>>,
    pub manual_liveliness_count: Option<Count>,
    pub lease_duration: Option<Duration>,
}

impl SDPBuiltinData {
    pub fn new() -> Self {
        Self::default()
    }

    fn from(
        domain_id: Option<u16>,
        domain_tag: Option<String>,
        protocol_version: Option<ProtocolVersion>,
        guid: Option<GUID>,
        vendor_id: Option<VendorId>,
        expects_inline_qos: Option<bool>,
        available_builtin_endpoint: Option<u32>,
        metarraffic_unicast_locator_list: Option<Vec<Locator>>,
        metarraffic_multicast_locator_list: Option<Vec<Locator>>,
        default_unicast_locator_list: Option<Vec<Locator>>,
        default_multicast_locator_list: Option<Vec<Locator>>,
        manual_liveliness_count: Option<Count>,
        lease_duration: Option<Duration>,
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

    pub fn toSPDPdiscoverdParticipantData(&mut self) -> SPDPdiscoveredParticipantData {
        let domain_id = self.domain_id.unwrap();
        let domain_tag = self.domain_tag.take().unwrap();
        let protocol_version = self.protocol_version.take().unwrap();
        let guid = self.guid.unwrap();
        let vendor_id = self.vendor_id.unwrap();
        let expects_inline_qos = self.expects_inline_qos.unwrap();
        let available_builtin_endpoint = self.available_builtin_endpoint.unwrap();
        let metarraffic_unicast_locator_list =
            self.metarraffic_unicast_locator_list.take().unwrap();
        let metarraffic_multicast_locator_list =
            self.metarraffic_multicast_locator_list.take().unwrap();
        let default_unicast_locator_list = self.default_unicast_locator_list.take().unwrap();
        let default_multicast_locator_list = self.default_multicast_locator_list.take().unwrap();
        let manual_liveliness_count = self.manual_liveliness_count.unwrap();
        let lease_duration = self.lease_duration.unwrap();

        SPDPdiscoveredParticipantData {
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

impl<'de> Deserialize<'de> for SDPBuiltinData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
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
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;
                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a RTPS serialized_payload")
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        let value = 256 * v[0] as u16 + v[1] as u16;
                        let pid = ParameterId { value };
                        match pid {
                            // ParameterId::PID_DOMAIN => Ok(Field::domain_id),
                            // ParameterId::PID_PARTICIPANT_GUID => Ok(Field::domain_tag),
                            ParameterId::PID_PROTOCOL_VERSION => Ok(Field::protocol_version),
                            ParameterId::PID_PARTICIPANT_GUID => Ok(Field::guid),
                            ParameterId::PID_VENDOR_ID => Ok(Field::vendor_id),
                            ParameterId::PID_EXPECTS_INLINE_QOS => Ok(Field::expects_inline_qos),
                            ParameterId::PID_BUILTIN_ENDPOINT_SET => {
                                Ok(Field::available_builtin_endpoint)
                            }
                            ParameterId::PID_METATRAFFIC_UNICAST_LOCATOR => {
                                Ok(Field::metarraffic_unicast_locator_list)
                            }
                            ParameterId::PID_METATRAFFIC_MULTICAST_LOCATOR => {
                                Ok(Field::metarraffic_multicast_locator_list)
                            }
                            ParameterId::PID_DEFAULT_UNICAST_LOCATOR => {
                                Ok(Field::metarraffic_unicast_locator_list)
                            }
                            ParameterId::PID_DEFAULT_MULTICAST_LOCATOR => {
                                Ok(Field::metarraffic_multicast_locator_list)
                            }
                            ParameterId::PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT => {
                                Ok(Field::manual_liveliness_count)
                            }
                            ParameterId::PID_PARTICIPANT_LEASE_DURATION => {
                                Ok(Field::lease_duration)
                            }
                            _ => Err(de::Error::unknown_field("hoge", FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }
        struct SDPBuiltinDataVisitor;

        impl<'de> Visitor<'de> for SDPBuiltinDataVisitor {
            type Value = SDPBuiltinData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct SPDPdiscoveredParticipantData")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<SDPBuiltinData, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let domain_id: Option<u16> = Some(0);
                let domain_tag: Option<String> = Some(String::from("hoge"));
                let mut protocol_version: Option<ProtocolVersion> = None;
                let mut guid: Option<GUID> = None;
                let mut vendor_id: Option<VendorId> = None;
                let mut expects_inline_qos: Option<bool> = None;
                let mut available_builtin_endpoint: Option<u32> = None;
                let mut metarraffic_unicast_locator_list: Option<Vec<Locator>> = Some(Vec::new());
                let mut metarraffic_multicast_locator_list: Option<Vec<Locator>> = Some(Vec::new());
                let mut default_unicast_locator_list: Option<Vec<Locator>> = Some(Vec::new());
                let mut default_multicast_locator_list: Option<Vec<Locator>> = Some(Vec::new());
                let mut manual_liveliness_count: Option<Count> = None;
                let mut lease_duration: Option<Duration> = None;

                loop {
                    let pid: u16 = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                    eprintln!(">>>pid: {:04X}", pid);
                    let parameter_id = ParameterId { value: pid };
                    let _length: u16 = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                    eprintln!(">>>length: {:04X}", _length);
                    match parameter_id {
                        ParameterId::PID_PROTOCOL_VERSION => {
                            protocol_version = seq.next_element()?;
                            eprintln!(">>>>>>protocol_version: {:?}", protocol_version);
                            let _pad: u16 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            assert_eq!(_pad, 0);
                        }
                        ParameterId::PID_PARTICIPANT_GUID => {
                            guid = seq.next_element()?;
                            eprintln!(">>>>>>guid: {:?}", guid);
                        }
                        ParameterId::PID_VENDOR_ID => {
                            vendor_id = seq.next_element()?;
                            eprintln!(">>>>>>vendor_id: {:?}", vendor_id);
                            let _pad: u16 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            assert_eq!(_pad, 0);
                        }
                        ParameterId::PID_EXPECTS_INLINE_QOS => {
                            expects_inline_qos = seq.next_element()?;
                            eprintln!(">>>>>>expects_inline_qos: {:?}", expects_inline_qos);
                            let _pad: u16 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            assert_eq!(_pad, 0);
                        }
                        ParameterId::PID_BUILTIN_ENDPOINT_SET => {
                            available_builtin_endpoint = seq.next_element()?;
                            eprintln!(
                                ">>>>>>available_builtin_endpoint: {:?}",
                                available_builtin_endpoint
                            );
                        }
                        /*
                            rtps spec 2.4
                            9.4.2.10 LocatorListはLocatorListのCDR encodingを以下のように示している。
                            LocatorList:
                            0...2...........8...............16.............24.
                            ............................................................... 31
                            +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
                            |                     unsigned long numLocators                 |
                            +---------------+---------------+---------------+---------------+
                            |                        Locator_t locator_1                    |
                            ~                               ...                             ~
                            |                       Locator_t locator_numLocators           |
                            +---------------+---------------+---------------+---------------+

                            実際のDDS実装(FastDDS [RTPS 2.1], RustDDS [RTPS 2.3])において、
                            locatorが1つしか含まれない場合のLocatorListのencodingは以下のようになっている。
                            LocatorList:
                            0...2...........8...............16.............24.
                            ............................................................... 31
                            +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
                            +---------------+---------------+---------------+---------------+
                            |                        Locator_t locator_1                    |
                            +---------------+---------------+---------------+---------------+
                            また、同実装において、LocatorListにLocatorが複数含まれる場合、
                            1つのLocatorに対し、1つのLocatorをエンコードし、SerializedPayloadに
                            複数の*_*_LOCATORが含まれるようにエンコードされている。

                        */
                        ParameterId::PID_METATRAFFIC_UNICAST_LOCATOR => {
                            let kind: i32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            let port: u32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            let address: [u8; 16] = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            match &mut metarraffic_unicast_locator_list {
                                Some(v) => {
                                    v.push(Locator::new(kind, port, address));
                                }
                                None => unreachable!(),
                            }
                            eprintln!(
                                ">>>>>>metarraffic_unicast_locator_list: {:?}",
                                metarraffic_unicast_locator_list
                            );
                        }
                        ParameterId::PID_METATRAFFIC_MULTICAST_LOCATOR => {
                            let kind: i32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            let port: u32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            let address: [u8; 16] = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            match &mut metarraffic_multicast_locator_list {
                                Some(v) => {
                                    v.push(Locator::new(kind, port, address));
                                }
                                None => unreachable!(),
                            }
                            eprintln!(
                                ">>>>>>metarraffic_multicast_locator_list: {:?}",
                                metarraffic_multicast_locator_list
                            );
                        }
                        ParameterId::PID_DEFAULT_UNICAST_LOCATOR => {
                            let kind: i32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            let port: u32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            let address: [u8; 16] = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            match &mut default_unicast_locator_list {
                                Some(v) => {
                                    v.push(Locator::new(kind, port, address));
                                }
                                None => unreachable!(),
                            }
                            eprintln!(
                                ">>>>>>default_unicast_locator_list: {:?}",
                                default_unicast_locator_list
                            );
                        }
                        ParameterId::PID_DEFAULT_MULTICAST_LOCATOR => {
                            let kind: i32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            let port: u32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            let address: [u8; 16] = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            match &mut default_multicast_locator_list {
                                Some(v) => {
                                    v.push(Locator::new(kind, port, address));
                                }
                                None => unreachable!(),
                            }
                            eprintln!(
                                ">>>>>>default_multicast_locator_list: {:?}",
                                default_multicast_locator_list
                            );
                        }
                        ParameterId::PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT => {
                            manual_liveliness_count = seq.next_element()?;
                            eprintln!(
                                ">>>>>>manual_liveliness_count: {:?}",
                                manual_liveliness_count
                            );
                        }
                        ParameterId::PID_PARTICIPANT_LEASE_DURATION => {
                            lease_duration = seq.next_element()?;
                            eprintln!(">>>>>>lease_duration: {:?}", lease_duration);
                        }
                        ParameterId::PID_SENTINEL => {
                            break;
                        }
                        _ => unimplemented!(),
                    }
                    eprintln!();
                }

                Ok(SDPBuiltinData::from(
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
                ))
            }

            /*
            fn visit_map<V>(self, mut map: V) -> Result<SDPBuiltinData, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut guid = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::guid => {
                            if guid.is_some() {
                                return Err(de::Error::duplicate_field("guid"));
                            }
                        }
                        ui => return Err(de::Error::duplicate_field("guid")),
                    }
                }
                let guid = guid.ok_or_else(|| de::Error::missing_field("guid"))?;
                Ok(SDPBuiltinData::from(todo!()))
            }
            */
        }
        const FIELDS: &'static [&'static str] = &["secs", "nanos"];
        deserializer.deserialize_struct("SDPBuiltinData", FIELDS, SDPBuiltinDataVisitor)
    }
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
        for metarraffic_unicast_locator in &self.metarraffic_unicast_locator_list {
            s.serialize_field(
                "parameterId",
                &ParameterId::PID_METATRAFFIC_UNICAST_LOCATOR.value,
            )?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field(
                "metarraffic_unicast_locator_list",
                metarraffic_unicast_locator,
            )?;
        }

        // metarraffic_multicast_locator_list
        for metarraffic_multicast_locator in &self.metarraffic_multicast_locator_list {
            s.serialize_field(
                "parameterId",
                &ParameterId::PID_METATRAFFIC_MULTICAST_LOCATOR.value,
            )?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field(
                "metarraffic_multicast_locator_list",
                metarraffic_multicast_locator,
            )?;
        }

        // default_unicast_locator_list
        for default_unicast_locator in &self.default_unicast_locator_list {
            s.serialize_field(
                "parameterId",
                &ParameterId::PID_DEFAULT_UNICAST_LOCATOR.value,
            )?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field("default_unicast_locator_list", default_unicast_locator)?;
        }

        // default_multicast_locator_list
        for default_multicast_locator in &self.default_multicast_locator_list {
            s.serialize_field(
                "parameterId",
                &ParameterId::PID_DEFAULT_MULTICAST_LOCATOR.value,
            )?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field("default_multicast_locator_list", default_multicast_locator)?;
        }

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
    use super::super::cdr::deserialize;
    use super::*;
    use crate::message::message_header::ProtocolVersion;
    use crate::message::submessage::element::{RepresentationIdentifier, SerializedPayload};
    use cdr::{Infinite, PlCdrLe};

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
        eprintln!("{}", serialized);
    }

    #[test]
    fn test_deserialize() {
        // 現状エラーを吐かずにデシリアライズできるかしかtestできてない。
        // TODO: デシリアライズしたdataとシリアライズ元のdataを比較
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
        let serialized = cdr::serialize::<_, _, PlCdrLe>(&data, Infinite).unwrap();
        let mut deseriarized = match deserialize::<SDPBuiltinData>(&serialized) {
            Ok(d) => d,
            Err(e) => panic!("neko~~~~~: failed deserialize\n{}", e),
        };
        let new_data = deseriarized.toSPDPdiscoverdParticipantData();
        eprintln!("domain_id: {}", new_data.domain_id);
        eprintln!("domain_tag: {}", new_data.domain_tag);
        eprintln!("protocol_version: {:?}", new_data.protocol_version);
        eprintln!("guid: {:?}", new_data.protocol_version);
        eprintln!("vendor_id: {:?}", new_data.vendor_id);
        eprintln!("expects_inline_qos: {:?}", new_data.expects_inline_qos);
        eprintln!(
            "available_builtin_endpoint: {:?}",
            new_data.available_builtin_endpoint
        );
        eprintln!(
            "metarraffic_unicast_locator_list: {:?}",
            new_data.metarraffic_unicast_locator_list
        );
        eprintln!(
            "metarraffic_multicast_locator_list: {:?}",
            new_data.metarraffic_multicast_locator_list
        );
        eprintln!(
            "default_unicast_locator_list: {:?}",
            new_data.default_unicast_locator_list
        );
        eprintln!(
            "default_multicast_locator_list: {:?}",
            new_data.default_multicast_locator_list
        );
        eprintln!(
            "manual_liveliness_count: {:?}",
            new_data.manual_liveliness_count
        );
        eprintln!("lease_duration: {:?}", new_data.lease_duration);
    }
}
