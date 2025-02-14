use crate::dds::qos::{policy::*, DataReaderQosBuilder, DataWriterQosBuilder};
use crate::discovery::structure::builtin_endpoint::BuiltinEndpoint;
use crate::message::message_header::ProtocolVersion;
use crate::message::submessage::element::{Count, Locator};
use crate::rtps::cache::HistoryCache;
use crate::structure::{Duration, ParameterId, ReaderProxy, VendorId, WriterProxy, GUID};
use alloc::sync::Arc;
use awkernel_sync::rwlock::RwLock;
use colored::*;
use enumflags2::BitFlags;
use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use serde::{ser::SerializeStruct, Serialize};
use std::fmt;

/// rtps spec, 9.6.2.1 Data Representation for the ParticipantMessageData Built-in Endpoints
#[derive(Clone, Serialize, serde::Deserialize)]
pub struct ParticipantMessageData {
    pub guid: GUID,
    pub kind: ParticipantMessageKind,
    pub data: Vec<u8>,
}
impl ParticipantMessageData {
    pub fn new(guid: GUID, kind: ParticipantMessageKind, data: Vec<u8>) -> Self {
        Self { guid, kind, data }
    }
}

#[derive(PartialEq, Clone, Serialize, serde::Deserialize)]
pub struct ParticipantMessageKind {
    pub value: [u8; 4],
}
impl ParticipantMessageKind {
    pub const UNKNOWN: Self = Self {
        value: [0x00, 0x00, 0x00, 0x00],
    };
    pub const AUTOMATIC_LIVELINESS_UPDATE: Self = Self {
        value: [0x00, 0x00, 0x00, 0x01],
    };
    pub const MANUAL_LIVELINESS_UPDATE: Self = Self {
        value: [0x00, 0x00, 0x00, 0x02],
    };
}

#[allow(dead_code)]
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

    // {Subscription/Publication}BuiltinTopicData
    pub type_name: Option<String>,
    pub topic_name: Option<String>,
    pub key: Option<()>,
    pub publication_key: Option<()>,

    pub durability: Option<Durability>,
    pub deadline: Option<Deadline>,
    pub latency_budget: Option<LatencyBudget>,
    pub liveliness: Option<Liveliness>,
    pub reliability: Option<Reliability>,
    pub user_data: Option<UserData>,
    pub ownership: Option<Ownership>,
    pub destination_order: Option<DestinationOrder>,
    pub time_based_filter: Option<TimeBasedFilter>,
    pub presentation: Option<Presentation>,
    pub partition: Option<Partition>,
    pub topic_data: Option<TopicData>,
    pub group_data: Option<GroupData>,
    pub durability_service: Option<DurabilityService>,
    pub lifespan: Option<Lifespan>,
    // PublicationBuiltinTopicData
    pub ownership_strength: Option<OwnershipStrength>,
}

impl SDPBuiltinData {
    #[allow(clippy::too_many_arguments)]
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
        type_name: Option<String>,
        topic_name: Option<String>,
        key: Option<()>,
        publication_key: Option<()>,
        durability: Option<Durability>,
        deadline: Option<Deadline>,
        latency_budget: Option<LatencyBudget>,
        liveliness: Option<Liveliness>,
        reliability: Option<Reliability>,
        user_data: Option<UserData>,
        ownership: Option<Ownership>,
        destination_order: Option<DestinationOrder>,
        time_based_filter: Option<TimeBasedFilter>,
        presentation: Option<Presentation>,
        partition: Option<Partition>,
        topic_data: Option<TopicData>,
        group_data: Option<GroupData>,
        durability_service: Option<DurabilityService>,
        lifespan: Option<Lifespan>,
        ownership_strength: Option<OwnershipStrength>,
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
            type_name,
            topic_name,
            key,
            publication_key,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability,
            user_data,
            ownership,
            destination_order,
            time_based_filter,
            presentation,
            partition,
            topic_data,
            group_data,
            durability_service,
            lifespan,
            ownership_strength,
        }
    }

    pub fn gen_spdp_discoverd_participant_data(&mut self) -> Option<SPDPdiscoveredParticipantData> {
        let domain_id = self.domain_id?; // TODO: set  domain_id of this participant if domain_id is none
        let _domain_tag = self.domain_tag.take().unwrap_or(String::from(""));
        let protocol_version = self.protocol_version.take()?;
        let guid = self.guid?;
        let vendor_id = self.vendor_id?;
        let expects_inline_qos = self.expects_inline_qos.unwrap_or(false);
        let available_builtin_endpoint = self.available_builtin_endpoint?;
        let metarraffic_unicast_locator_list = self.metarraffic_unicast_locator_list.take()?;
        let metarraffic_multicast_locator_list = self.metarraffic_multicast_locator_list.take()?;
        let default_unicast_locator_list = self.default_unicast_locator_list.take()?;
        let default_multicast_locator_list = self.default_multicast_locator_list.take()?;
        let manual_liveliness_count = self.manual_liveliness_count;
        let lease_duration = self.lease_duration.unwrap_or(Duration {
            seconds: 100,
            fraction: 0,
        });

        Some(SPDPdiscoveredParticipantData {
            domain_id,
            _domain_tag,
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
        })
    }

    pub fn topic_info(&self) -> Option<(String, String)> {
        let name = match &self.topic_name {
            Some(n) => n.clone(),
            None => return None,
        };
        let data_type = match &self.type_name {
            Some(d) => d.clone(),
            None => return None,
        };
        Some((name, data_type))
    }

    pub fn gen_readerpoxy(
        &mut self,
        history_cache: Arc<RwLock<HistoryCache>>,
    ) -> Option<ReaderProxy> {
        let remote_guid = self.remote_guid?;
        let expects_inline_qos = self.expects_inline_qos.unwrap_or(false);
        let unicast_locator_list = self.unicast_locator_list.take()?;
        let multicast_locator_list = self.multicast_locator_list.take()?;
        let dr_qos_builder = DataReaderQosBuilder::new();
        let qos = dr_qos_builder
            .durability(self.durability.unwrap_or_default())
            .deadline(self.deadline.unwrap_or_default())
            .latency_budget(self.latency_budget.unwrap_or_default())
            .liveliness(self.liveliness.unwrap_or_default())
            .reliability(
                self.reliability
                    .unwrap_or(Reliability::default_besteffort()),
            )
            .destination_order(self.destination_order.unwrap_or_default())
            .user_data(self.user_data.clone().unwrap_or_default())
            .ownership(self.ownership.unwrap_or_default())
            .time_based_filter(self.time_based_filter.unwrap_or_default())
            .build();
        Some(ReaderProxy::new(
            remote_guid,
            expects_inline_qos,
            unicast_locator_list,
            multicast_locator_list,
            qos,
            history_cache,
        ))
    }

    pub fn gen_writerproxy(
        &mut self,
        history_cache: Arc<RwLock<HistoryCache>>,
    ) -> Option<WriterProxy> {
        let remote_guid = match self.remote_guid {
            Some(rg) => rg,
            None => {
                eprintln!(
                    "<{}>: couldn't gen WriterProxy, not found remote_guid",
                    "SDPBuiltinData: Err".red()
                );
                return None;
            }
        };
        let unicast_locator_list = match self.unicast_locator_list.take() {
            Some(ull) => ull,
            None => {
                eprintln!(
                    "<{}>: couldn't gen WriterProxy, not found unicast_locator_list",
                    "SDPBuiltinData: Err".red()
                );
                return None;
            }
        };
        let multicast_locator_list = match self.multicast_locator_list.take() {
            Some(mll) => mll,
            None => {
                eprintln!(
                    "<{}>: couldn't gen WriterProxy, not found multicast_locator_list",
                    "SDPBuiltinData: Error".red()
                );
                return None;
            }
        };
        let data_max_size_serialized = self.data_max_size_serialized.unwrap_or(0); // TODO: Which default value should I set?
        let dw_qos_builder = DataWriterQosBuilder::new();
        let qos = dw_qos_builder
            .durability(self.durability.unwrap_or_default())
            .durability_service(self.durability_service.unwrap_or_default())
            .deadline(self.deadline.unwrap_or_default())
            .latency_budget(self.latency_budget.unwrap_or_default())
            .liveliness(self.liveliness.unwrap_or_default())
            .reliability(self.reliability.unwrap_or(Reliability::default_reliable()))
            .destination_order(self.destination_order.unwrap_or_default())
            .user_data(self.user_data.clone().unwrap_or_default())
            .ownership(self.ownership.unwrap_or_default())
            .ownership_strength(self.ownership_strength.unwrap_or_default())
            .lifespan(self.lifespan.unwrap_or_default())
            .build();
        Some(WriterProxy::new(
            remote_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
            qos,
            history_cache,
        ))
    }
    // rtps 2.4 spec: 8.5.4.4 Data Types associated with built-in Endpoints used by the Simple Endpoint Discovery Protocol
    // An implementation of the protocol need not necessarily send all information contained in the DataTypes.
    // If any information is not present, the implementation can assume the default values, as defined by the PSM.
    // MEMO: SEDPのbuilt-in
    // dataへの変換を実装するとき、足りないデータがあれば、デフォルト値でおぎなう。
    // デフォルト値はrtps 2.3 spec "Table 9.14 - ParameterId mapping and default values"にある。
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Property {
    name: String,
    value: String,
}

impl<'de> Deserialize<'de> for SDPBuiltinData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[allow(dead_code)]
        enum Field {
            DomainID,
            DomainTag,
            ProtocolVersion,
            Guid,
            VendorId,
            ExpectsInlineQos,
            AvailableBuiltinEndpoint,
            MetatrafficUnicastLocatorList,
            MetatrafficMulticastLocatorList,
            DefaultUnicastLocatorList,
            DefaultMulticastLocatorList,
            ManualLivelinessCount,
            LeaseDuration,
            RemoteGuid,
            UnicastLocatorList,
            MulticastLocatorList,
            DataMaxSizeSerialized,
            TypeName,
            TopicName,
            Key,
            PublicationKey,
            Durability,
            Deadline,
            LatencyBudget,
            Liveliness,
            Reliability,
            UserData,
            Ownership,
            DestinationOrder,
            TimeBasedFilter,
            Presentation,
            Partition,
            TopicData,
            GroupData,
            DurabilityService,
            Lifespan,
            OwnershipStrength,
        }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl Visitor<'_> for FieldVisitor {
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
                            ParameterId::PID_DOMAIN_ID => Ok(Field::DomainID),
                            ParameterId::PID_DOMAIN_TAG => Ok(Field::DomainTag),
                            ParameterId::PID_PROTOCOL_VERSION => Ok(Field::ProtocolVersion),
                            ParameterId::PID_PARTICIPANT_GUID => Ok(Field::Guid),
                            ParameterId::PID_VENDOR_ID => Ok(Field::VendorId),
                            ParameterId::PID_EXPECTS_INLINE_QOS => Ok(Field::ExpectsInlineQos),
                            ParameterId::PID_BUILTIN_ENDPOINT_SET => {
                                Ok(Field::AvailableBuiltinEndpoint)
                            }
                            ParameterId::PID_METATRAFFIC_UNICAST_LOCATOR => {
                                Ok(Field::MetatrafficUnicastLocatorList)
                            }
                            ParameterId::PID_METATRAFFIC_MULTICAST_LOCATOR => {
                                Ok(Field::MetatrafficMulticastLocatorList)
                            }
                            ParameterId::PID_DEFAULT_UNICAST_LOCATOR => {
                                Ok(Field::DefaultUnicastLocatorList)
                            }
                            ParameterId::PID_DEFAULT_MULTICAST_LOCATOR => {
                                Ok(Field::DefaultMulticastLocatorList)
                            }
                            ParameterId::PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT => {
                                Ok(Field::ManualLivelinessCount)
                            }
                            ParameterId::PID_PARTICIPANT_LEASE_DURATION => Ok(Field::LeaseDuration),
                            ParameterId::PID_ENDPOINT_GUID => Ok(Field::RemoteGuid),
                            ParameterId::PID_UNICAST_LOCATOR => Ok(Field::UnicastLocatorList),
                            ParameterId::PID_MULTICAST_LOCATOR => Ok(Field::MulticastLocatorList),
                            ParameterId::PID_TYPE_MAX_SIZE_SERIALIZED => {
                                Ok(Field::DataMaxSizeSerialized)
                            }
                            ParameterId::PID_TYPE_NAME => Ok(Field::TypeName),
                            ParameterId::PID_TOPIC_NAME => Ok(Field::TopicName),
                            ParameterId::PID_DURABILITY => Ok(Field::Durability),
                            ParameterId::PID_DEADLINE => Ok(Field::Deadline),
                            ParameterId::PID_LATENCY_BUDGET => Ok(Field::LatencyBudget),
                            ParameterId::PID_LIVELINESS => Ok(Field::Liveliness),
                            ParameterId::PID_RELIABILITY => Ok(Field::Reliability),
                            ParameterId::PID_USER_DATA => Ok(Field::UserData),
                            ParameterId::PID_OWNERSHIP => Ok(Field::Ownership),
                            ParameterId::PID_DESTINATION_ORDER => Ok(Field::DestinationOrder),
                            ParameterId::PID_TIME_BASED_FILTER => Ok(Field::TimeBasedFilter),
                            ParameterId::PID_PRESENTATION => Ok(Field::Presentation),
                            ParameterId::PID_PARTITION => Ok(Field::Partition),
                            ParameterId::PID_TOPIC_DATA => Ok(Field::TopicData),
                            ParameterId::PID_GROUP_DATA => Ok(Field::GroupData),
                            ParameterId::PID_DURABILITY_SERVICE => Ok(Field::DurabilityService),
                            ParameterId::PID_LIFESPAN => Ok(Field::Lifespan),
                            ParameterId::PID_OWNERSHIP_STRENGTH => Ok(Field::OwnershipStrength),
                            pid => Err(de::Error::unknown_field(
                                &format!("pid: {:?}", pid.value),
                                FIELDS,
                            )),
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
                let mut domain_tag: Option<String> = None;
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
                let mut type_name: Option<String> = None;
                let mut topic_name: Option<String> = None;
                let key: Option<()> = None;
                let publication_key: Option<()> = None;
                let mut durability: Option<Durability> = None;
                let mut deadline: Option<Deadline> = None;
                let mut latency_budget: Option<LatencyBudget> = None;
                let mut liveliness: Option<Liveliness> = None;
                let mut reliability: Option<Reliability> = None;
                let mut user_data: Option<UserData> = None;
                let mut ownership: Option<Ownership> = None;
                let mut destination_order: Option<DestinationOrder> = None;
                let mut time_based_filter: Option<TimeBasedFilter> = None;
                let mut presentation: Option<Presentation> = None;
                let mut partition: Option<Partition> = None;
                let mut topic_data: Option<TopicData> = None;
                let mut group_data: Option<GroupData> = None;
                let mut durability_service: Option<DurabilityService> = None;
                let mut lifespan: Option<Lifespan> = None;
                let mut ownership_strength: Option<OwnershipStrength> = None;

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
                    if pid == 0 {
                        // pid 0 is PID_PAD
                        continue;
                    }
                    let parameter_id = ParameterId { value: pid };
                    let length: u16 = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                    match parameter_id {
                        ParameterId::PID_DOMAIN_ID => {
                            domain_id = seq.next_element()?;
                            read_pad!(u16);
                        }
                        ParameterId::PID_DOMAIN_TAG => {
                            domain_tag = seq.next_element()?;
                        }
                        ParameterId::PID_ENTITY_NAME => {
                            // for compatibility between this implementation and FastDDS
                            let _entity_name: Option<String> = seq.next_element()?;
                        }
                        ParameterId::PID_PROTOCOL_VERSION => {
                            protocol_version = seq.next_element()?;
                            read_pad!(u16);
                        }
                        ParameterId::PID_PARTICIPANT_GUID => {
                            guid = seq.next_element()?;
                        }
                        ParameterId::PID_VENDOR_ID => {
                            vendor_id = seq.next_element()?;
                            read_pad!(u16);
                        }
                        ParameterId::PID_EXPECTS_INLINE_QOS => {
                            expects_inline_qos = seq.next_element()?;
                            read_pad!(u16);
                        }
                        ParameterId::PID_BUILTIN_ENDPOINT_SET => {
                            available_builtin_endpoint = seq.next_element()?;
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
                        }
                        ParameterId::PID_METATRAFFIC_MULTICAST_LOCATOR => {
                            read_locator_list!(metarraffic_multicast_locator_list);
                        }
                        ParameterId::PID_DEFAULT_UNICAST_LOCATOR => {
                            read_locator_list!(default_unicast_locator_list);
                        }
                        ParameterId::PID_DEFAULT_MULTICAST_LOCATOR => {
                            read_locator_list!(default_multicast_locator_list);
                        }
                        ParameterId::PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT => {
                            manual_liveliness_count = seq.next_element()?;
                        }
                        ParameterId::PID_PARTICIPANT_LEASE_DURATION => {
                            lease_duration = seq.next_element()?;
                        }
                        ParameterId::PID_ENDPOINT_GUID => {
                            remote_guid = seq.next_element()?;
                        }
                        ParameterId::PID_UNICAST_LOCATOR => {
                            read_locator_list!(unicast_locator_list);
                        }
                        ParameterId::PID_MULTICAST_LOCATOR => {
                            read_locator_list!(multicast_locator_list);
                        }
                        ParameterId::PID_TYPE_MAX_SIZE_SERIALIZED => {
                            data_max_size_serialized = seq.next_element()?;
                        }
                        ParameterId::PID_TYPE_NAME => {
                            type_name = seq.next_element()?;
                        }
                        ParameterId::PID_TOPIC_NAME => {
                            topic_name = seq.next_element()?;
                        }
                        ParameterId::PID_DURABILITY => {
                            durability = seq.next_element()?;
                        }
                        ParameterId::PID_DEADLINE => {
                            deadline = seq.next_element()?;
                        }
                        ParameterId::PID_LATENCY_BUDGET => {
                            latency_budget = seq.next_element()?;
                        }
                        ParameterId::PID_LIVELINESS => {
                            liveliness = seq.next_element()?;
                        }
                        ParameterId::PID_RELIABILITY => {
                            reliability = seq.next_element()?;
                        }
                        ParameterId::PID_USER_DATA => {
                            user_data = seq.next_element()?;
                        }
                        ParameterId::PID_OWNERSHIP => {
                            ownership = seq.next_element()?;
                        }
                        ParameterId::PID_DESTINATION_ORDER => {
                            destination_order = seq.next_element()?;
                        }
                        ParameterId::PID_TIME_BASED_FILTER => {
                            time_based_filter = seq.next_element()?;
                        }
                        ParameterId::PID_PRESENTATION => {
                            presentation = seq.next_element()?;
                            read_pad!(u16);
                        }
                        ParameterId::PID_PARTITION => {
                            partition = seq.next_element()?;
                        }
                        ParameterId::PID_TOPIC_DATA => {
                            topic_data = seq.next_element()?;
                        }
                        ParameterId::PID_GROUP_DATA => {
                            group_data = seq.next_element()?;
                        }
                        ParameterId::PID_DURABILITY_SERVICE => {
                            durability_service = seq.next_element()?;
                        }
                        ParameterId::PID_LIFESPAN => {
                            lifespan = seq.next_element()?;
                        }
                        ParameterId::PID_OWNERSHIP_STRENGTH => {
                            ownership_strength = seq.next_element()?;
                        }
                        ParameterId::PID_PROPERTY_LIST => {
                            // for compatibility between this implementation and FastDDS
                            let _property_list: Option<Vec<Property>> = seq.next_element()?;
                        }
                        ParameterId::PID_SENTINEL => {
                            break;
                        }
                        _ => {
                            let mut l = length;
                            while l > 0 {
                                l -= 1;
                                let _something: u8 = seq
                                    .next_element()?
                                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            }
                            eprintln!(
                                "<{}>: unimplemented ParameterId: 0x{:04X} received",
                                "SDPBuiltinData: Warn".yellow(),
                                pid
                            );
                        }
                    }
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
                    type_name,
                    topic_name,
                    key,
                    publication_key,
                    durability,
                    deadline,
                    latency_budget,
                    liveliness,
                    reliability,
                    user_data,
                    ownership,
                    destination_order,
                    time_based_filter,
                    presentation,
                    partition,
                    topic_data,
                    group_data,
                    durability_service,
                    lifespan,
                    ownership_strength,
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
        const FIELDS: &[&str] = &["secs", "nanos"];
        deserializer.deserialize_struct("SDPBuiltinData", FIELDS, SDPBuiltinDataVisitor)
    }
}

#[derive(Clone)]
pub struct SubscriptionBuiltinTopicData {
    pub key: Option<()>,
    pub publication_key: Option<()>,
    pub topic_name: Option<String>,
    pub type_name: Option<String>,
    pub durability: Option<Durability>,
    pub deadline: Option<Deadline>,
    pub latency_budget: Option<LatencyBudget>,
    pub liveliness: Option<Liveliness>,
    pub reliability: Option<Reliability>,
    pub ownership: Option<Ownership>,
    pub destination_order: Option<DestinationOrder>,
    pub user_data: Option<UserData>,
    pub time_based_filter: Option<TimeBasedFilter>,
    pub presentation: Option<Presentation>,
    pub partition: Option<Partition>,
    pub topic_data: Option<TopicData>,
    pub group_data: Option<GroupData>,
    pub durability_service: Option<DurabilityService>,
    pub lifespan: Option<Lifespan>,
}
impl SubscriptionBuiltinTopicData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: Option<()>,
        publication_key: Option<()>,
        topic_name: Option<String>,
        type_name: Option<String>,
        durability: Option<Durability>,
        deadline: Option<Deadline>,
        latency_budget: Option<LatencyBudget>,
        liveliness: Option<Liveliness>,
        reliability: Option<Reliability>,
        ownership: Option<Ownership>,
        destination_order: Option<DestinationOrder>,
        user_data: Option<UserData>,
        time_based_filter: Option<TimeBasedFilter>,
        presentation: Option<Presentation>,
        partition: Option<Partition>,
        topic_data: Option<TopicData>,
        group_data: Option<GroupData>,
        durability_service: Option<DurabilityService>,
        lifespan: Option<Lifespan>,
    ) -> Self {
        Self {
            key,
            publication_key,
            topic_name,
            type_name,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability,
            ownership,
            destination_order,
            user_data,
            time_based_filter,
            presentation,
            partition,
            topic_data,
            group_data,
            durability_service,
            lifespan,
        }
    }
}

impl Serialize for SubscriptionBuiltinTopicData {
    // serialization format of String is wrong
    // TODO: fix it
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("SubscriptionBuiltinTopicData", 4)?;
        // topic_name
        if let Some(topic_name) = &self.topic_name {
            s.serialize_field("parameterId", &ParameterId::PID_TOPIC_NAME.value)?;
            let str_len = topic_name.len() as u16;
            let serialized_str_len = str_len + 4 + 1; // 4 is length of string, 0 is null
            let pad_len = (4 - serialized_str_len % 4) % 4;
            s.serialize_field::<u16>("parameterLength", &(serialized_str_len + pad_len))?;
            s.serialize_field("topic_name", &topic_name)?;
            for _ in 0..pad_len {
                s.serialize_field::<u8>("padding", &0)?;
            }
        }

        // type_name
        if let Some(type_name) = &self.type_name {
            s.serialize_field("parameterId", &ParameterId::PID_TYPE_NAME.value)?;
            let str_len = type_name.len() as u16;
            let serialized_str_len = str_len + 4 + 1; // 4 is length of string, 0 is null
            let pad_len = (4 - serialized_str_len % 4) % 4;
            s.serialize_field::<u16>("parameterLength", &(serialized_str_len + pad_len))?;
            s.serialize_field("type_name", &type_name)?;
            for _ in 0..pad_len {
                s.serialize_field::<u8>("padding", &0)?;
            }
        }

        /*
        // durability
        if let Some(durability) = &self.durability {
            s.serialize_field("parameterId", &ParameterId::PID_DURABILITY.value)?;
            s.serialize_field::<u16>("parameterLength", &4)?;
            s.serialize_field("durability", &durability)?;
        }

        // deadline
        if let Some(deadline) = &self.deadline {
            s.serialize_field("parameterId", &ParameterId::PID_DEADLINE.value)?;
            s.serialize_field::<u16>("parameterLength", &8)?;
            s.serialize_field("deadline", &deadline)?;
        }

        // latency_budget
        if let Some(latency_budget) = &self.latency_budget {
            s.serialize_field("parameterId", &ParameterId::PID_LATENCY_BUDGET.value)?;
            s.serialize_field::<u16>("parameterLength", &8)?;
            s.serialize_field("latency_budget", &latency_budget)?;
        }

        // liveliness
        if let Some(liveliness) = &self.liveliness {
            s.serialize_field("parameterId", &ParameterId::PID_LIVELINESS.value)?;
            s.serialize_field::<u16>("parameterLength", &12)?;
            s.serialize_field("liveliness", &liveliness)?;
        }

        // reliability
        if let Some(reliability) = &self.reliability {
            s.serialize_field("parameterId", &ParameterId::PID_RELIABILITY.value)?;
            s.serialize_field::<u16>("parameterLength", &12)?;
            s.serialize_field("reliability", &reliability)?;
        }

        // ownership
        if let Some(ownership) = &self.ownership {
            s.serialize_field("parameterId", &ParameterId::PID_OWNERSHIP.value)?;
            s.serialize_field::<u16>("parameterLength", &4)?;
            s.serialize_field("ownership", &ownership)?;
        }

        // destination_order
        if let Some(destination_order) = &self.destination_order {
            s.serialize_field("parameterId", &ParameterId::PID_DESTINATION_ORDER.value)?;
            s.serialize_field::<u16>("parameterLength", &4)?;
            s.serialize_field("destination_order", &destination_order)?;
        }

        // user_data
        if let Some(user_data) = &self.user_data {
            s.serialize_field("parameterId", &ParameterId::PID_USER_DATA.value)?;
            s.serialize_field::<u16>("parameterLength", &user_data.serialized_size())?;
            s.serialize_field("user_data", &user_data)?;
        }

        // time_based_filter
        if let Some(time_based_filter) = &self.time_based_filter {
            s.serialize_field("parameterId", &ParameterId::PID_TIME_BASED_FILTER.value)?;
            s.serialize_field::<u16>("parameterLength", &8)?;
            s.serialize_field("time_based_filter", &time_based_filter)?;
        }

        // presentation
        if let Some(presentation) = &self.presentation {
            s.serialize_field("parameterId", &ParameterId::PID_PRESENTATION.value)?;
            s.serialize_field::<u16>("parameterLength", &8)?;
            s.serialize_field("presentation", &presentation)?;
            s.serialize_field::<u16>("vendorId: padding", &0)?;
        }

        // partition
        if let Some(partition) = &self.partition {
            s.serialize_field("parameterId", &ParameterId::PID_PARTITION.value)?;
            s.serialize_field::<u16>("parameterLength", &partition.serialized_size())?;
            s.serialize_field("partition", &partition)?;
        }

        // topic_data
        if let Some(topic_data) = &self.topic_data {
            s.serialize_field("parameterId", &ParameterId::PID_TOPIC_DATA.value)?;
            s.serialize_field::<u16>("parameterLength", &topic_data.serialized_size())?;
            s.serialize_field("topic_data", &topic_data)?;
        }

        // group_data
        if let Some(group_data) = &self.group_data {
            s.serialize_field("parameterId", &ParameterId::PID_GROUP_DATA.value)?;
            s.serialize_field::<u16>("parameterLength", &group_data.serialized_size())?;
            s.serialize_field("group_data", &group_data)?;
        }

        // durability_service
        if let Some(durability_service) = &self.durability_service {
            s.serialize_field("parameterId", &ParameterId::PID_DURABILITY_SERVICE.value)?;
            s.serialize_field::<u16>("parameterLength", &28)?;
            s.serialize_field("durability_service", &durability_service)?;
        }

        // lifespan
        if let Some(lifespan) = &self.lifespan {
            s.serialize_field("parameterId", &ParameterId::PID_LIFESPAN.value)?;
            s.serialize_field::<u16>("parameterLength", &8)?;
            s.serialize_field("lifespan", &lifespan)?;
        }
        */

        s.end()
    }
}

#[derive(Clone)]
pub struct PublicationBuiltinTopicData {
    pub key: Option<()>,
    pub publication_key: Option<()>,
    pub topic_name: Option<String>,
    pub type_name: Option<String>,
    pub durability: Option<Durability>,
    pub durability_service: Option<DurabilityService>,
    pub deadline: Option<Deadline>,
    pub latency_budget: Option<LatencyBudget>,
    pub liveliness: Option<Liveliness>,
    pub reliability: Option<Reliability>,
    pub lifespan: Option<Lifespan>,
    pub user_data: Option<UserData>,
    pub time_based_filter: Option<TimeBasedFilter>,
    pub ownership: Option<Ownership>,
    pub ownership_strength: Option<OwnershipStrength>,
    pub destination_order: Option<DestinationOrder>,
    pub presentation: Option<Presentation>,
    pub partition: Option<Partition>,
    pub topic_data: Option<TopicData>,
    pub group_data: Option<GroupData>,
}
impl PublicationBuiltinTopicData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: Option<()>,
        publication_key: Option<()>,
        topic_name: Option<String>,
        type_name: Option<String>,
        durability: Option<Durability>,
        durability_service: Option<DurabilityService>,
        deadline: Option<Deadline>,
        latency_budget: Option<LatencyBudget>,
        liveliness: Option<Liveliness>,
        reliability: Option<Reliability>,
        lifespan: Option<Lifespan>,
        user_data: Option<UserData>,
        time_based_filter: Option<TimeBasedFilter>,
        ownership: Option<Ownership>,
        ownership_strength: Option<OwnershipStrength>,
        destination_order: Option<DestinationOrder>,
        presentation: Option<Presentation>,
        partition: Option<Partition>,
        topic_data: Option<TopicData>,
        group_data: Option<GroupData>,
    ) -> Self {
        Self {
            key,
            publication_key,
            topic_name,
            type_name,
            durability,
            durability_service,
            deadline,
            latency_budget,
            liveliness,
            reliability,
            lifespan,
            user_data,
            time_based_filter,
            ownership,
            ownership_strength,
            destination_order,
            presentation,
            partition,
            topic_data,
            group_data,
        }
    }
}
impl Serialize for PublicationBuiltinTopicData {
    // serialization format of String is wrong
    // TODO: fix it
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("PublicationBuiltinTopicData", 4)?;
        // type_name
        if let Some(type_name) = &self.type_name {
            s.serialize_field("parameterId", &ParameterId::PID_TYPE_NAME.value)?;
            let str_len = type_name.len() as u16;
            let serialized_str_len = str_len + 4 + 1; // 4 is length of string, 0 is null
            let pad_len = (4 - serialized_str_len % 4) % 4;
            s.serialize_field::<u16>("parameterLength", &(serialized_str_len + pad_len))?;
            s.serialize_field("type_name", &type_name)?;
            for _ in 0..pad_len {
                s.serialize_field::<u8>("padding", &0)?;
            }
        }

        // topic_name
        if let Some(topic_name) = &self.topic_name {
            s.serialize_field("parameterId", &ParameterId::PID_TOPIC_NAME.value)?;
            let str_len = topic_name.len() as u16;
            let serialized_str_len = str_len + 4 + 1; // 4 is length of string, 0 is null
            let pad_len = (4 - serialized_str_len % 4) % 4;
            s.serialize_field::<u16>("parameterLength", &(serialized_str_len + pad_len))?;
            s.serialize_field("topic_name", &topic_name)?;
            for _ in 0..pad_len {
                s.serialize_field::<u8>("padding", &0)?;
            }
        }

        /*
        // durability
        if let Some(durability) = &self.durability {
            s.serialize_field("parameterId", &ParameterId::PID_DURABILITY.value)?;
            s.serialize_field::<u16>("parameterLength", &4)?;
            s.serialize_field("durability", &durability)?;
        }

        // durability_service
        if let Some(durability_service) = &self.durability_service {
            s.serialize_field("parameterId", &ParameterId::PID_DURABILITY_SERVICE.value)?;
            s.serialize_field::<u16>("parameterLength", &28)?;
            s.serialize_field("durability_service", &durability_service)?;
        }

        // deadline
        if let Some(deadline) = &self.deadline {
            s.serialize_field("parameterId", &ParameterId::PID_DEADLINE.value)?;
            s.serialize_field::<u16>("parameterLength", &8)?;
            s.serialize_field("deadline", &deadline)?;
        }

        // latency_budget
        if let Some(latency_budget) = &self.latency_budget {
            s.serialize_field("parameterId", &ParameterId::PID_LATENCY_BUDGET.value)?;
            s.serialize_field::<u16>("parameterLength", &8)?;
            s.serialize_field("latency_budget", &latency_budget)?;
        }

        // liveliness
        if let Some(liveliness) = &self.liveliness {
            s.serialize_field("parameterId", &ParameterId::PID_LIVELINESS.value)?;
            s.serialize_field::<u16>("parameterLength", &12)?;
            s.serialize_field("liveliness", &liveliness)?;
        }

        // reliability
        if let Some(reliability) = &self.reliability {
            s.serialize_field("parameterId", &ParameterId::PID_RELIABILITY.value)?;
            s.serialize_field::<u16>("parameterLength", &12)?;
            s.serialize_field("reliability", &reliability)?;
        }

        // lifespan
        if let Some(lifespan) = &self.lifespan {
            s.serialize_field("parameterId", &ParameterId::PID_LIFESPAN.value)?;
            s.serialize_field::<u16>("parameterLength", &8)?;
            s.serialize_field("lifespan", &lifespan)?;
        }

        // user_data
        if let Some(user_data) = &self.user_data {
            s.serialize_field("parameterId", &ParameterId::PID_USER_DATA.value)?;
            s.serialize_field::<u16>("parameterLength", &(4 + user_data.value.len() as u16))?;
            s.serialize_field("user_data", &user_data)?;
        }

        // time_based_filter
        if let Some(time_based_filter) = &self.time_based_filter {
            s.serialize_field("parameterId", &ParameterId::PID_TIME_BASED_FILTER.value)?;
            s.serialize_field::<u16>("parameterLength", &8)?;
            s.serialize_field("time_based_filter", &time_based_filter)?;
        }

        // ownership
        if let Some(ownership) = &self.ownership {
            s.serialize_field("parameterId", &ParameterId::PID_OWNERSHIP.value)?;
            s.serialize_field::<u16>("parameterLength", &4)?;
            s.serialize_field("ownership", &ownership)?;
        }

        // ownership_strength
        if let Some(ownership_strength) = &self.ownership_strength {
            s.serialize_field("parameterId", &ParameterId::PID_OWNERSHIP_STRENGTH.value)?;
            s.serialize_field::<u16>("parameterLength", &4)?;
            s.serialize_field("ownership_strength", &ownership_strength)?;
        }

        // destination_order
        if let Some(destination_order) = &self.destination_order {
            s.serialize_field("parameterId", &ParameterId::PID_DESTINATION_ORDER.value)?;
            s.serialize_field::<u16>("parameterLength", &4)?;
            s.serialize_field("destination_order", &destination_order)?;
        }

        // presentation
        if let Some(presentation) = &self.presentation {
            s.serialize_field("parameterId", &ParameterId::PID_PRESENTATION.value)?;
            s.serialize_field::<u16>("parameterLength", &8)?;
            s.serialize_field("presentation", &presentation)?;
            s.serialize_field::<u16>("vendorId: padding", &0)?;
        }

        // partition
        if let Some(partition) = &self.partition {
            s.serialize_field("parameterId", &ParameterId::PID_PARTITION.value)?;
            s.serialize_field::<u16>("parameterLength", &partition.serialized_size())?;
            s.serialize_field("partition", &partition)?;
        }

        // topic_data
        if let Some(topic_data) = &self.topic_data {
            s.serialize_field("parameterId", &ParameterId::PID_TOPIC_DATA.value)?;
            s.serialize_field::<u16>("parameterLength", &topic_data.serialized_size())?;
            s.serialize_field("topic_data", &topic_data)?;
        }

        // group_data
        if let Some(group_data) = &self.group_data {
            s.serialize_field("parameterId", &ParameterId::PID_GROUP_DATA.value)?;
            s.serialize_field::<u16>("parameterLength", &group_data.serialized_size())?;
            s.serialize_field("group_data", &group_data)?;
        }
        */

        s.end()
    }
}

#[derive(Clone)]
pub struct DiscoveredReaderData {
    proxy: ReaderProxy,
    builtin_topic_data: SubscriptionBuiltinTopicData,
}
impl DiscoveredReaderData {
    pub fn new(proxy: ReaderProxy, builtin_topic_data: SubscriptionBuiltinTopicData) -> Self {
        Self {
            proxy,
            builtin_topic_data,
        }
    }
}
impl Serialize for DiscoveredReaderData {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("DiscoveredReaderData", 2)?;
        // proxy
        s.serialize_field("proxy", &self.proxy)?;

        // SubscriptionBuiltinTopicData
        s.serialize_field("builtin_topic_data", &self.builtin_topic_data)?;

        // sentinel
        s.serialize_field("parameterId", &ParameterId::PID_SENTINEL.value)?;
        s.serialize_field::<u16>("vendorId: padding", &0)?;

        s.end()
    }
}

#[derive(Clone)]
pub struct DiscoveredWriterData {
    proxy: WriterProxy,
    builtin_topic_data: PublicationBuiltinTopicData,
}
impl DiscoveredWriterData {
    pub fn new(proxy: WriterProxy, builtin_topic_data: PublicationBuiltinTopicData) -> Self {
        Self {
            proxy,
            builtin_topic_data,
        }
    }
}
impl Serialize for DiscoveredWriterData {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("DiscoveredWriterData", 2)?;
        // proxy
        s.serialize_field("proxy", &self.proxy)?;

        // SubscriptionBuiltinTopicData
        s.serialize_field("builtin_topic_data", &self.builtin_topic_data)?;

        // sentinel
        s.serialize_field("parameterId", &ParameterId::PID_SENTINEL.value)?;
        s.serialize_field::<u16>("vendorId: padding", &0)?;

        s.end()
    }
}

#[derive(Clone)]
pub struct SPDPdiscoveredParticipantData {
    pub domain_id: u16,
    pub _domain_tag: String,
    pub protocol_version: ProtocolVersion,
    pub guid: GUID,
    pub vendor_id: VendorId,
    pub expects_inline_qos: bool,
    pub available_builtin_endpoint: BitFlags<BuiltinEndpoint>, // parameter_id: ParameterId::PID_BUILTIN_ENDPOINT_SET
    pub metarraffic_unicast_locator_list: Vec<Locator>,
    pub metarraffic_multicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub manual_liveliness_count: Option<Count>, // Some DDS implementation's SPDP message don't contains manual_liveliness_count, rtps 2.3 spec don't specify default value of manual_liveliness_count.
    pub lease_duration: Duration,
}

impl SPDPdiscoveredParticipantData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain_id: u16,
        _domain_tag: String,
        protocol_version: ProtocolVersion,
        guid: GUID,
        vendor_id: VendorId,
        expects_inline_qos: bool,
        available_builtin_endpoint: BitFlags<BuiltinEndpoint>,
        metarraffic_unicast_locator_list: Vec<Locator>,
        metarraffic_multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        manual_liveliness_count: Option<Count>,
        lease_duration: Duration,
    ) -> Self {
        Self {
            domain_id,
            _domain_tag,
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
        if let Some(manual_liveliness_count) = self.manual_liveliness_count {
            s.serialize_field(
                "parameterId",
                &ParameterId::PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT.value,
            )?;
            s.serialize_field::<u16>("parameterLength", &4)?;
            s.serialize_field::<i32>("manual_liveliness_count", &manual_liveliness_count)?;
        }

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
    use enumflags2::make_bitflags;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn test_serialize() {
        let mut small_rng = SmallRng::seed_from_u64(0);

        // 現状シリアライズ結果を既存実装のキャプチャと目視で比較しかできない
        // TODO: シリアライズ済のバイナリをソースに埋めて、シリアライズ結果と比較
        let data = SPDPdiscoveredParticipantData::new(
            0,
            "hoge".to_string(),
            ProtocolVersion::PROTOCOLVERSION,
            GUID::new_participant_guid(&mut small_rng),
            VendorId::THIS_IMPLEMENTATION,
            false,
            make_bitflags!(BuiltinEndpoint::{DISC_BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER}),
            Locator::new_list_from_self_ipv4(7400),
            Locator::new_list_from_self_ipv4(7410),
            Locator::new_list_from_self_ipv4(7411),
            Locator::new_list_from_self_ipv4(7440),
            Some(0),
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
        let mut small_rng = SmallRng::seed_from_u64(0);

        // 現状エラーを吐かずにデシリアライズできるかしかtestできてない。
        // TODO: デシリアライズしたdataとシリアライズ元のdataを比較
        let data = SPDPdiscoveredParticipantData::new(
            0,
            "hoge".to_string(),
            ProtocolVersion::PROTOCOLVERSION,
            GUID::new_participant_guid(&mut small_rng),
            VendorId::THIS_IMPLEMENTATION,
            false,
            make_bitflags!(BuiltinEndpoint::{DISC_BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER}),
            Locator::new_list_from_self_ipv4(7400),
            Locator::new_list_from_self_ipv4(7410),
            Locator::new_list_from_self_ipv4(7411),
            Locator::new_list_from_self_ipv4(7440),
            Some(0),
            Duration {
                seconds: 20,
                fraction: 0,
            },
        );
        let serialized =
            cdr::serialize::<_, _, PlCdrLe>(&data, Infinite).expect("couldn't serialize message");
        let mut deseriarized = match deserialize::<SDPBuiltinData>(&serialized) {
            Ok(d) => d,
            Err(e) => panic!("failed deserialize\n{}", e),
        };
        let new_data = deseriarized
            .gen_spdp_discoverd_participant_data()
            .expect("couldn't get spdp data from SDPBuiltinData");
        eprintln!("domain_id: {}", new_data.domain_id);
        eprintln!("domain_tag: {}", new_data._domain_tag);
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
