use crate::discovery::structure::builtin_endpoint::BuiltinEndpoint;
use crate::message::message_header::ProtocolVersion;
use crate::message::submessage::element::{Count, Locator, Parameter};
use crate::structure::duration::Duration;
use crate::structure::{
    guid::{GuidPrefix, GUID},
    parameter_id::ParameterId,
    proxy::{ReaderProxy, WriterProxy},
    vendor_id::VendorId,
};
use enumflags2::{bitflags, make_bitflags, BitFlags};
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::fmt;

#[derive(Clone, Default)]
pub struct SDPBuiltinData {
    // SPDPdiscoveredParticipantData
    pub domain_id: Option<u16>,
    pub domain_tag: Option<String>,
    pub protocol_version: Option<ProtocolVersion>,
    pub guid: Option<GUID>,
    pub vendor_id: Option<VendorId>,
    pub expects_inline_qos: Option<bool>,
    pub available_builtin_endpoint: Option<BitFlags<BuiltinEndpoint>>, // parameter_id: ParameterId::PID_BUILTIN_ENDPOINT_SET
    pub metarraffic_unicast_locator_list: Option<Vec<Locator>>,
    pub metarraffic_multicast_locator_list: Option<Vec<Locator>>,
    pub default_multicast_locator_list: Option<Vec<Locator>>,
    pub default_unicast_locator_list: Option<Vec<Locator>>,
    pub manual_liveliness_count: Option<Count>,
    pub lease_duration: Option<Duration>,

    // {Reader/Writer}Proxy
    pub remote_guid: Option<GUID>,
    pub unicast_locator_list: Option<Vec<Locator>>,
    pub multicast_locator_list: Option<Vec<Locator>>,
    // expects_inline_qos // only ReaderProxy
    pub data_max_size_serialized: Option<i32>, // only ReaderProxy

                                               // SubscriptionBuiltinTopicData
                                               // PublicationBuiltinTopicData
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
        available_builtin_endpoint: Option<BitFlags<BuiltinEndpoint>>,
        metarraffic_unicast_locator_list: Option<Vec<Locator>>,
        metarraffic_multicast_locator_list: Option<Vec<Locator>>,
        default_unicast_locator_list: Option<Vec<Locator>>,
        default_multicast_locator_list: Option<Vec<Locator>>,
        manual_liveliness_count: Option<Count>,
        lease_duration: Option<Duration>,
        remote_guid: Option<GUID>,
        unicast_locator_list: Option<Vec<Locator>>,
        multicast_locator_list: Option<Vec<Locator>>,
        data_max_size_serialized: Option<i32>,
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
            remote_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
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

    fn toReaderProxy(&mut self) -> ReaderProxy {
        let remote_guid = self.remote_guid.unwrap();
        let expects_inline_qos = self.expects_inline_qos.unwrap();
        let unicast_locator_list = self.unicast_locator_list.take().unwrap();
        let multicast_locator_list = self.multicast_locator_list.take().unwrap();
        ReaderProxy::new(
            remote_guid,
            expects_inline_qos,
            unicast_locator_list,
            multicast_locator_list,
        )
    }

    fn toWriterProxy(&mut self) -> WriterProxy {
        let remote_guid = self.remote_guid.unwrap();
        let unicast_locator_list = self.unicast_locator_list.take().unwrap();
        let multicast_locator_list = self.multicast_locator_list.take().unwrap();
        let data_max_size_serialized = self.data_max_size_serialized.unwrap();
        WriterProxy::new(
            remote_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
        )
    }
    // rtps 2.4 spec: 8.5.4.4 Data Types associated with built-in Endpoints used by the Simple Endpoint Discovery Protocol
    // An implementation of the protocol need not necessarily send all information contained in the DataTypes.
    // If any information is not present, the implementation can assume the default values, as defined by the PSM.
    // MEMO: SEDPのbuilt-in
    // dataへの変換を実装するとき、足りないデータがあれば、デフォルト値でおぎなう。
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
            remote_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
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
                                Ok(Field::default_unicast_locator_list)
                            }
                            ParameterId::PID_DEFAULT_MULTICAST_LOCATOR => {
                                Ok(Field::default_multicast_locator_list)
                            }
                            ParameterId::PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT => {
                                Ok(Field::manual_liveliness_count)
                            }
                            ParameterId::PID_PARTICIPANT_LEASE_DURATION => {
                                Ok(Field::lease_duration)
                            }
                            ParameterId::PID_ENDPOINT_GUID => Ok(Field::remote_guid),
                            ParameterId::PID_UNICAST_LOCATOR => Ok(Field::unicast_locator_list),
                            ParameterId::PID_MULTICAST_LOCATOR => Ok(Field::multicast_locator_list),
                            ParameterId::PID_TYPE_MAX_SIZE_SERIALIZED => {
                                Ok(Field::data_max_size_serialized)
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
                let mut domain_id: Option<u16> = Some(0);
                let mut domain_tag: Option<String> = Some(String::from("hoge"));
                let mut protocol_version: Option<ProtocolVersion> = None;
                let mut guid: Option<GUID> = None;
                let mut vendor_id: Option<VendorId> = None;
                let mut expects_inline_qos: Option<bool> = None;
                let mut available_builtin_endpoint: Option<BitFlags<BuiltinEndpoint>> = None;
                let mut metarraffic_unicast_locator_list: Option<Vec<Locator>> = Some(Vec::new());
                let mut metarraffic_multicast_locator_list: Option<Vec<Locator>> = Some(Vec::new());
                let mut default_unicast_locator_list: Option<Vec<Locator>> = Some(Vec::new());
                let mut default_multicast_locator_list: Option<Vec<Locator>> = Some(Vec::new());
                let mut manual_liveliness_count: Option<Count> = None;
                let mut lease_duration: Option<Duration> = None;
                let mut remote_guid: Option<GUID> = None;
                let mut unicast_locator_list: Option<Vec<Locator>> = Some(Vec::new());
                let mut multicast_locator_list: Option<Vec<Locator>> = Some(Vec::new());
                let mut data_max_size_serialized: Option<i32> = None;

                macro_rules! read_pad {
                    ($type:ty) => {{
                        let _pad: $type = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        assert_eq!(_pad, 0);
                    }};
                }
                macro_rules! read_locator_list {
                    ($ll:ident) => {{
                        let kind: i32 = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        let port: u32 = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        let address: [u8; 16] = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        match &mut $ll {
                            Some(v) => {
                                v.push(Locator::new(kind, port, address));
                            }
                            None => unreachable!(),
                        }
                    }};
                }
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
                        ParameterId::PID_DOMAIN_ID => {
                            domain_id = seq.next_element()?;
                            eprintln!(">>>>>>domain_id: {:?}", domain_id);
                            read_pad!(u16);
                        }
                        ParameterId::PID_DOMAIN_TAG => {
                            domain_tag = seq.next_element()?;
                            eprintln!(">>>>>>domain_tag: {:?}", domain_tag);
                        }
                        ParameterId::PID_PROTOCOL_VERSION => {
                            protocol_version = seq.next_element()?;
                            eprintln!(">>>>>>protocol_version: {:?}", protocol_version);
                            read_pad!(u16);
                        }
                        ParameterId::PID_PARTICIPANT_GUID => {
                            guid = seq.next_element()?;
                            eprintln!(">>>>>>guid: {:?}", guid);
                        }
                        ParameterId::PID_VENDOR_ID => {
                            vendor_id = seq.next_element()?;
                            eprintln!(">>>>>>vendor_id: {:?}", vendor_id);
                            read_pad!(u16);
                        }
                        ParameterId::PID_EXPECTS_INLINE_QOS => {
                            expects_inline_qos = seq.next_element()?;
                            eprintln!(">>>>>>expects_inline_qos: {:?}", expects_inline_qos);
                            read_pad!(u16);
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
                            read_locator_list!(metarraffic_unicast_locator_list);
                            eprintln!(
                                ">>>>>>metarraffic_unicast_locator_list: {:?}",
                                metarraffic_unicast_locator_list
                            );
                        }
                        ParameterId::PID_METATRAFFIC_MULTICAST_LOCATOR => {
                            read_locator_list!(metarraffic_multicast_locator_list);
                            eprintln!(
                                ">>>>>>metarraffic_multicast_locator_list: {:?}",
                                metarraffic_multicast_locator_list
                            );
                        }
                        ParameterId::PID_DEFAULT_UNICAST_LOCATOR => {
                            read_locator_list!(default_unicast_locator_list);
                            eprintln!(
                                ">>>>>>default_unicast_locator_list: {:?}",
                                default_unicast_locator_list
                            );
                        }
                        ParameterId::PID_DEFAULT_MULTICAST_LOCATOR => {
                            read_locator_list!(default_multicast_locator_list);
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
                        ParameterId::PID_ENDPOINT_GUID => {
                            remote_guid = seq.next_element()?;
                            eprintln!(">>>>>>remote_guid: {:?}", remote_guid);
                        }
                        ParameterId::PID_UNICAST_LOCATOR => {
                            read_locator_list!(unicast_locator_list);
                            eprintln!(">>>>>>unicast_locator_list: {:?}", unicast_locator_list);
                        }
                        ParameterId::PID_MULTICAST_LOCATOR => {
                            read_locator_list!(multicast_locator_list);
                            eprintln!(">>>>>>multicast_locator_list: {:?}", multicast_locator_list);
                        }
                        ParameterId::PID_TYPE_MAX_SIZE_SERIALIZED => {
                            data_max_size_serialized = seq.next_element()?;
                            eprintln!(
                                ">>>>>>data_max_size_serialized: {:?}",
                                data_max_size_serialized
                            );
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
                    remote_guid,
                    unicast_locator_list,
                    multicast_locator_list,
                    data_max_size_serialized,
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
pub struct SubscriptionBuiltinTopicData {
    // TODO
}

#[derive(Clone)]
pub struct PublicationBuiltinTopicData {
    // TODO
}

#[derive(Clone)]
pub struct SPDPdiscoveredParticipantData {
    pub domain_id: u16,
    pub domain_tag: String,
    pub protocol_version: ProtocolVersion,
    pub guid: GUID,
    pub vendor_id: VendorId,
    pub expects_inline_qos: bool,
    pub available_builtin_endpoint: BitFlags<BuiltinEndpoint>, // parameter_id: ParameterId::PID_BUILTIN_ENDPOINT_SET
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
        available_builtin_endpoint: BitFlags<BuiltinEndpoint>,
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
            make_bitflags!(BuiltinEndpoint::{DISC_BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER}),
            Locator::new_list_from_self_ipv4(7400),
            Locator::new_list_from_self_ipv4(7410),
            Locator::new_list_from_self_ipv4(7411),
            Locator::new_list_from_self_ipv4(7440),
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
            make_bitflags!(BuiltinEndpoint::{DISC_BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER}),
            Locator::new_list_from_self_ipv4(7400),
            Locator::new_list_from_self_ipv4(7410),
            Locator::new_list_from_self_ipv4(7411),
            Locator::new_list_from_self_ipv4(7440),
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
