use crate::dds::key::KeyHash;
use crate::dds::qos::{policy::*, DataReaderQosBuilder, DataWriterQosBuilder};
use crate::discovery::structure::builtin_endpoint::BuiltinEndpoint;
use crate::message::message_header::ProtocolVersion;
use crate::message::submessage::element::{Count, Locator};
use crate::rtps::cache::HistoryCache;
use crate::structure::{
    Duration, GuidPrefix, ParameterId, ReaderProxy, VendorId, WriterProxy, GUID,
};
use crate::utils::pad_len;
use crate::DdsData;
use alloc::sync::Arc;
use awkernel_sync::rwlock::RwLock;
use cdr::{CdrBe, Infinite};
use enumflags2::BitFlags;
use log::{trace, warn};
use md5::compute;

/// rtps spec, 8.4.13.4 Data Types Associated with Built-in Endpoints used by Writer Liveliness Protocol
/// rtps spec, 9.6.2.1 Data Representation for the ParticipantMessageData Built-in Endpoints
/// The former shows that the first member of ParticipantMessageData is GUID, but the later said
/// that the first member of ParticipantMessageData is serialized to GuidPrefix.
/// You can watch Participant of Cyclone DDS sending ParticipantMessageData with GuidPrefix.
/// This implementation send ParticipantMessageData with GuidPrefix.
#[derive(Clone, DdsData)]
pub struct ParticipantMessageData {
    #[key]
    // To serialize the first member of ParticipantMessageData to GuidPrefix.
    pub guid_prefix: GuidPrefix,
    pub kind: ParticipantMessageKind,
    pub data: Vec<u8>,
}
impl ParticipantMessageData {
    pub fn new(guid_prefix: GuidPrefix, kind: ParticipantMessageKind, data: Vec<u8>) -> Self {
        Self {
            guid_prefix,
            kind,
            data,
        }
    }
}

impl<C: speedy::Context> speedy::Writable<C> for ParticipantMessageData {
    #[inline]
    fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_value(&self.guid_prefix)?;
        writer.write_value(&self.kind)?;
        let data_len = self.data.len();
        writer.write_u32(data_len as u32)?;
        writer.write_bytes(&self.data)?;

        // padding
        const PADDING: [u8; 3] = [0, 0, 0];
        writer.write_bytes(&PADDING[..pad_len(data_len)])?;

        Ok(())
    }
}

impl<'a, C: speedy::Context> speedy::Readable<'a, C> for ParticipantMessageData {
    #[inline]
    fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let guid_prefix = reader.read_value()?;
        let kind = reader.read_value()?;
        let data_len = reader.read_u32()? as usize;
        let data = reader.read_vec(data_len)?;

        // padding
        reader.skip_bytes(pad_len(data_len))?;

        Ok(ParticipantMessageData {
            guid_prefix,
            kind,
            data,
        })
    }
}

#[derive(PartialEq, Clone)]
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

impl<C: speedy::Context> speedy::Writable<C> for ParticipantMessageKind {
    #[inline]
    fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_bytes(&self.value)?;

        Ok(())
    }
}

impl<'a, C: speedy::Context> speedy::Readable<'a, C> for ParticipantMessageKind {
    #[inline]
    fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let mut value = [0; 4];
        reader.read_bytes(&mut value)?;

        Ok(Self { value })
    }
}

#[allow(dead_code)]
#[derive(Clone, Default, DdsData)]
pub struct SDPBuiltinData {
    // SPDPdiscoveredParticipantData
    pub domain_id: Option<u16>,
    pub domain_tag: Option<String>,
    pub protocol_version: Option<ProtocolVersion>,
    #[key]
    pub guid: Option<GUID>,
    pub vendor_id: Option<VendorId>,
    pub expects_inline_qos: Option<bool>,
    pub available_builtin_endpoint: Option<BitFlags<BuiltinEndpoint>>, // parameter_id: ParameterId::PID_BUILTIN_ENDPOINT_SET
    pub metarraffic_unicast_locator_list: Vec<Locator>,
    pub metarraffic_multicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub manual_liveliness_count: Option<Count>,
    pub lease_duration: Option<Duration>,

    // {Reader/Writer}Proxy
    pub remote_guid: Option<GUID>,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
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
    pub history: Option<History>,
    pub resource_limits: Option<ResourceLimits>,
    pub transport_priority: Option<TransportPriority>,
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
        metarraffic_unicast_locator_list: Vec<Locator>,
        metarraffic_multicast_locator_list: Vec<Locator>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        manual_liveliness_count: Option<Count>,
        lease_duration: Option<Duration>,
        remote_guid: Option<GUID>,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
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
        history: Option<History>,
        resource_limits: Option<ResourceLimits>,
        transport_priority: Option<TransportPriority>,
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
            history,
            resource_limits,
            transport_priority,
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
        let metarraffic_unicast_locator_list = self.metarraffic_unicast_locator_list.clone();
        let metarraffic_multicast_locator_list = self.metarraffic_multicast_locator_list.clone();
        let default_unicast_locator_list = self.default_unicast_locator_list.clone();
        let default_multicast_locator_list = self.default_multicast_locator_list.clone();
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

    pub fn topic_info(&self) -> Option<(&String, &String)> {
        let name = match &self.topic_name {
            Some(n) => n,
            None => return None,
        };
        let data_type = match &self.type_name {
            Some(d) => d,
            None => return None,
        };
        Some((name, data_type))
    }

    pub fn gen_readerpoxy(
        &mut self,
        history_cache: Arc<RwLock<HistoryCache>>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> Option<ReaderProxy> {
        let remote_guid = match self.remote_guid {
            Some(rg) => rg,
            None => {
                warn!("failed to gen ReaderProxy from SDPBuiltinData, not found remote_guid",);
                return None;
            }
        };
        let expects_inline_qos = self.expects_inline_qos.unwrap_or(false);
        let unicast_locator_list = self.unicast_locator_list.clone();
        let multicast_locator_list = self.multicast_locator_list.clone();
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
            .history(self.history.unwrap_or_default())
            .resource_limits(self.resource_limits.unwrap_or_default())
            .user_data(self.user_data.clone().unwrap_or_default())
            .ownership(self.ownership.unwrap_or_default())
            .time_based_filter(self.time_based_filter.unwrap_or_default())
            .build();
        Some(ReaderProxy::new(
            remote_guid,
            expects_inline_qos,
            unicast_locator_list,
            multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            qos,
            history_cache,
            true,
        ))
    }

    pub fn gen_writerproxy(
        &mut self,
        history_cache: Arc<RwLock<HistoryCache>>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> Option<WriterProxy> {
        let remote_guid = match self.remote_guid {
            Some(rg) => rg,
            None => {
                warn!("failed to gen WriterProxy from SDPBuiltinData, not found remote_guid",);
                return None;
            }
        };
        let unicast_locator_list = self.unicast_locator_list.clone();
        let multicast_locator_list = self.multicast_locator_list.clone();
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
            .history(self.history.unwrap_or_default())
            .resource_limits(self.resource_limits.unwrap_or_default())
            .transport_priority(self.transport_priority.unwrap_or_default())
            .user_data(self.user_data.clone().unwrap_or_default())
            .ownership(self.ownership.unwrap_or_default())
            .ownership_strength(self.ownership_strength.unwrap_or_default())
            .lifespan(self.lifespan.unwrap_or_default())
            .build();
        Some(WriterProxy::new(
            remote_guid,
            unicast_locator_list,
            multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
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
#[derive(Debug)]
struct Property {
    name: String,
    value: String,
}
impl<'a, C: speedy::Context> speedy::Readable<'a, C> for Property {
    #[inline]
    fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let name = {
            let cdr_name_len = reader.read_i32()?;
            let name = reader.read_string((cdr_name_len - 1) as usize)?;
            reader.read_u8()?; // null char
            reader.skip_bytes(pad_len(cdr_name_len as usize))?; // padding
            name
        };
        let value = {
            let cdr_value_len = reader.read_i32()?;
            let value = reader.read_string((cdr_value_len - 1) as usize)?;
            reader.read_u8()?; // null char
            reader.skip_bytes(pad_len(cdr_value_len as usize))?; // padding
            value
        };
        Ok(Self { name, value })
    }
}
impl<C: speedy::Context> speedy::Writable<C> for Property {
    #[inline]
    fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        let cdr_name_len = self.name.len() + 1;
        writer.write_i32(cdr_name_len as i32)?;
        writer.write_bytes(self.name.as_bytes())?;
        writer.write_u8(0)?; // null char

        // padding
        const ZEROS: [u8; 3] = [0; 3];
        writer.write_bytes(&ZEROS[..pad_len(cdr_name_len)])?;

        let cdr_value_len = self.value.len() + 1;
        writer.write_i32(cdr_value_len as i32)?;
        writer.write_bytes(self.value.as_bytes())?;
        writer.write_u8(0)?; // null char

        // padding
        writer.write_bytes(&ZEROS[..pad_len(cdr_value_len)])?;

        Ok(())
    }
}

impl<'a, C: speedy::Context> speedy::Readable<'a, C> for SDPBuiltinData {
    #[inline]
    fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let mut domain_id: Option<u16> = Some(0);
        let mut domain_tag: Option<String> = None;
        let mut protocol_version: Option<ProtocolVersion> = None;
        let mut guid: Option<GUID> = None;
        let mut vendor_id: Option<VendorId> = None;
        let mut expects_inline_qos: Option<bool> = None;
        let mut available_builtin_endpoint: Option<BitFlags<BuiltinEndpoint>> = None;
        let mut metarraffic_unicast_locator_list: Vec<Locator> = Vec::new();
        let mut metarraffic_multicast_locator_list: Vec<Locator> = Vec::new();
        let mut default_unicast_locator_list: Vec<Locator> = Vec::new();
        let mut default_multicast_locator_list: Vec<Locator> = Vec::new();
        let mut manual_liveliness_count: Option<Count> = None;
        let mut lease_duration: Option<Duration> = None;
        let mut remote_guid: Option<GUID> = None;
        let mut unicast_locator_list: Vec<Locator> = Vec::new();
        let mut multicast_locator_list: Vec<Locator> = Vec::new();
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
        let mut history: Option<History> = None;
        let mut resource_limits: Option<ResourceLimits> = None;
        let mut transport_priority: Option<TransportPriority> = None;
        let mut time_based_filter: Option<TimeBasedFilter> = None;
        let mut presentation: Option<Presentation> = None;
        let mut partition: Option<Partition> = None;
        let mut topic_data: Option<TopicData> = None;
        let mut group_data: Option<GroupData> = None;
        let mut durability_service: Option<DurabilityService> = None;
        let mut lifespan: Option<Lifespan> = None;
        let mut ownership_strength: Option<OwnershipStrength> = None;

        macro_rules! read_locator_list {
            ($ll:ident) => {{
                let kind: i32 = reader.read_value()?;
                let port: u32 = reader.read_value()?;
                let address: [u8; 16] = reader.read_value()?;
                $ll.push(Locator::new(kind, port, address));
            }};
        }

        loop {
            let pid = reader.read_u16()?;
            // println!("pid: 0x{:04x}", pid);
            if pid == 0 {
                // pid 0 is PID_PAD
                continue;
            }
            let parameter_id = ParameterId { value: pid };
            let length = reader.read_u16()?;

            match parameter_id {
                ParameterId::PID_DOMAIN_ID => {
                    domain_id = Some(reader.read_value()?);
                    reader.skip_bytes(2)?;
                }
                ParameterId::PID_DOMAIN_TAG => {
                    domain_tag = Some(reader.read_value()?);
                }
                ParameterId::PID_ENTITY_NAME => {
                    let _entity_name: String = reader.read_value()?;
                }
                ParameterId::PID_PROTOCOL_VERSION => {
                    protocol_version = Some(reader.read_value()?);
                    reader.skip_bytes(2)?;
                }
                ParameterId::PID_PARTICIPANT_GUID => {
                    guid = Some(reader.read_value()?);
                }
                ParameterId::PID_VENDOR_ID => {
                    vendor_id = Some(reader.read_value()?);
                    reader.skip_bytes(2)?;
                }
                ParameterId::PID_EXPECTS_INLINE_QOS => {
                    let flag = reader.read_u8()?;
                    assert!(flag == 1 || flag == 0);
                    expects_inline_qos = Some(flag == 1);
                    reader.skip_bytes(3)?;
                }
                ParameterId::PID_BUILTIN_ENDPOINT_SET => {
                    let bits = reader.read_u32()?;
                    available_builtin_endpoint =
                        Some(BitFlags::<BuiltinEndpoint>::from_bits_truncate(bits));
                }
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
                    manual_liveliness_count = Some(reader.read_value()?);
                }
                ParameterId::PID_PARTICIPANT_LEASE_DURATION => {
                    lease_duration = Some(reader.read_value()?);
                }
                ParameterId::PID_ENDPOINT_GUID => {
                    remote_guid = Some(reader.read_value()?);
                }
                ParameterId::PID_UNICAST_LOCATOR => {
                    read_locator_list!(unicast_locator_list);
                }
                ParameterId::PID_MULTICAST_LOCATOR => {
                    read_locator_list!(multicast_locator_list);
                }
                ParameterId::PID_TYPE_MAX_SIZE_SERIALIZED => {
                    data_max_size_serialized = Some(reader.read_value()?);
                }
                ParameterId::PID_TYPE_NAME => {
                    let cdr_str_len = reader.read_u32()?;
                    let bytes: Vec<u8> = reader.read_vec((cdr_str_len - 1) as usize)?;
                    reader.skip_bytes(1 + pad_len(cdr_str_len as usize))?; // null char + padding
                    type_name = Some(unsafe { String::from_utf8_unchecked(bytes) });
                }
                ParameterId::PID_TOPIC_NAME => {
                    let cdr_str_len = reader.read_u32()?;
                    let bytes: Vec<u8> = reader.read_vec((cdr_str_len - 1) as usize)?;
                    reader.skip_bytes(1 + pad_len(cdr_str_len as usize))?; // null char + padding
                    topic_name = Some(unsafe { String::from_utf8_unchecked(bytes) });
                }
                ParameterId::PID_DURABILITY => {
                    durability = Some(reader.read_value()?);
                }
                ParameterId::PID_DEADLINE => {
                    deadline = Some(reader.read_value()?);
                }
                ParameterId::PID_LATENCY_BUDGET => {
                    latency_budget = Some(reader.read_value()?);
                }
                ParameterId::PID_LIVELINESS => {
                    liveliness = Some(reader.read_value()?);
                }
                ParameterId::PID_RELIABILITY => {
                    reliability = Some(reader.read_value()?);
                }
                ParameterId::PID_USER_DATA => {
                    user_data = Some(reader.read_value()?);
                }
                ParameterId::PID_OWNERSHIP => {
                    ownership = Some(reader.read_value()?);
                }
                ParameterId::PID_DESTINATION_ORDER => {
                    destination_order = Some(reader.read_value()?);
                }
                ParameterId::PID_HISTORY => {
                    history = Some(reader.read_value()?);
                }
                ParameterId::PID_RESOURCE_LIMITS => {
                    resource_limits = Some(reader.read_value()?);
                }
                ParameterId::PID_TRANSPORT_PRIORITY => {
                    transport_priority = Some(reader.read_value()?);
                }
                ParameterId::PID_TIME_BASED_FILTER => {
                    time_based_filter = Some(reader.read_value()?);
                }
                ParameterId::PID_PRESENTATION => {
                    presentation = Some(reader.read_value()?);
                    reader.skip_bytes(2)?;
                }
                ParameterId::PID_PARTITION => {
                    partition = Some(reader.read_value()?);
                }
                ParameterId::PID_TOPIC_DATA => {
                    topic_data = Some(reader.read_value()?);
                }
                ParameterId::PID_GROUP_DATA => {
                    group_data = Some(reader.read_value()?);
                }
                ParameterId::PID_DURABILITY_SERVICE => {
                    durability_service = Some(reader.read_value()?);
                }
                ParameterId::PID_LIFESPAN => {
                    lifespan = Some(reader.read_value()?);
                }
                ParameterId::PID_OWNERSHIP_STRENGTH => {
                    ownership_strength = Some(reader.read_value()?);
                }
                ParameterId::PID_PROPERTY_LIST => {
                    let num_of_property = reader.read_u32()?;
                    for _ in 0..num_of_property {
                        let _property: Property = reader.read_value()?;
                    }
                }
                ParameterId::PID_SENTINEL => {
                    break;
                }
                _ => {
                    reader.skip_bytes(length as usize)?;
                    trace!(
                        "received DATA with ParameterList with unimplemented ParameterId: 0x{:04X}",
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
            history,
            resource_limits,
            transport_priority,
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

impl<C: speedy::Context> speedy::Writable<C> for SubscriptionBuiltinTopicData {
    #[inline]
    fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        // type_name
        if let Some(type_name) = &self.type_name {
            writer.write_u16(ParameterId::PID_TYPE_NAME.value)?;
            let str_len = type_name.len() as u16;
            let cdr_str_len = str_len + 4 + 1; // 4 is length of string, 1 is null char
            let pad_len = (4 - cdr_str_len % 4) % 4;
            let param_len = cdr_str_len + pad_len;
            writer.write_u16(param_len)?;

            // string length
            writer.write_u32((str_len + 1) as u32)?; // +1 is null char

            let bytes = type_name.as_bytes();
            writer.write_bytes(bytes)?;
            writer.write_u8(0)?; // null char

            const ZEROS: [u8; 3] = [0; 3];
            writer.write_bytes(&ZEROS[..pad_len as usize])?;
        }

        // topic_name
        if let Some(topic_name) = &self.topic_name {
            writer.write_u16(ParameterId::PID_TOPIC_NAME.value)?;
            let str_len = topic_name.len() as u16;
            let cdr_str_len = str_len + 4 + 1; // 4 is length of string, 1 is null char
            let pad_len = (4 - cdr_str_len % 4) % 4;
            let param_len = cdr_str_len + pad_len;
            writer.write_u16(param_len)?;

            // string length
            writer.write_u32((str_len + 1) as u32)?; // +1 is null char

            let bytes = topic_name.as_bytes();
            writer.write_bytes(bytes)?;
            writer.write_u8(0)?; // null char

            const ZEROS: [u8; 3] = [0; 3];
            writer.write_bytes(&ZEROS[..pad_len as usize])?;
        }

        Ok(())
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

impl<C: speedy::Context> speedy::Writable<C> for PublicationBuiltinTopicData {
    #[inline]
    fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        // type_name
        if let Some(type_name) = &self.type_name {
            writer.write_u16(ParameterId::PID_TYPE_NAME.value)?;
            let str_len = type_name.len() as u16;
            let cdr_str_len = str_len + 4 + 1; // 4 is length of string, 1 is null char
            let pad_len = (4 - cdr_str_len % 4) % 4;
            let param_len = cdr_str_len + pad_len;
            writer.write_u16(param_len)?;

            // string length
            writer.write_u32((str_len + 1) as u32)?; // +1 is null char

            let bytes = type_name.as_bytes();
            writer.write_bytes(bytes)?;
            writer.write_u8(0)?; // null char

            const ZEROS: [u8; 3] = [0; 3];
            writer.write_bytes(&ZEROS[..pad_len as usize])?;
        }

        // topic_name
        if let Some(topic_name) = &self.topic_name {
            writer.write_u16(ParameterId::PID_TOPIC_NAME.value)?;
            let str_len = topic_name.len() as u16;
            let cdr_str_len = str_len + 4 + 1; // 4 is length of string, 1 is null char
            let pad_len = (4 - cdr_str_len % 4) % 4;
            let param_len = cdr_str_len + pad_len;
            writer.write_u16(param_len)?;

            // string length
            writer.write_u32((str_len + 1) as u32)?; // +1 is null char

            let bytes = topic_name.as_bytes();
            writer.write_bytes(bytes)?;
            writer.write_u8(0)?; // null char

            const ZEROS: [u8; 3] = [0; 3];
            writer.write_bytes(&ZEROS[..pad_len as usize])?;
        }

        Ok(())
    }
}

#[derive(Clone, DdsData)]
pub struct DiscoveredReaderData {
    #[key]
    key: (),
    // Normally, we would use the key and publication_key from builtin_topic_data to compute the Key, but implementing this is difficult.
    // Since there's currently no need to compute the key, we are using this approach as a temporary solution.
    proxy: ReaderProxy,
    builtin_topic_data: SubscriptionBuiltinTopicData,
}
impl DiscoveredReaderData {
    pub fn new(proxy: ReaderProxy, builtin_topic_data: SubscriptionBuiltinTopicData) -> Self {
        Self {
            key: (),
            proxy,
            builtin_topic_data,
        }
    }
}

impl<C: speedy::Context> speedy::Writable<C> for DiscoveredReaderData {
    #[inline]
    fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_value(&self.proxy)?;
        writer.write_value(&self.builtin_topic_data)?;

        // sentinel
        writer.write_u16(ParameterId::PID_SENTINEL.value)?;
        writer.write_u16(0)?; // padding

        Ok(())
    }
}

#[derive(Clone, DdsData)]
pub struct DiscoveredWriterData {
    #[key]
    key: (),
    // Normally, we would use the key and publication_key from builtin_topic_data to compute the Key, but implementing this is difficult.
    // Since there's currently no need to compute the key, we are using this approach as a temporary solution.
    proxy: WriterProxy,
    builtin_topic_data: PublicationBuiltinTopicData,
}
impl DiscoveredWriterData {
    pub fn new(proxy: WriterProxy, builtin_topic_data: PublicationBuiltinTopicData) -> Self {
        Self {
            key: (),
            proxy,
            builtin_topic_data,
        }
    }
}

impl<C: speedy::Context> speedy::Writable<C> for DiscoveredWriterData {
    #[inline]
    fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_value(&self.proxy)?;
        writer.write_value(&self.builtin_topic_data)?;

        // sentinel
        writer.write_u16(ParameterId::PID_SENTINEL.value)?;
        writer.write_u16(0)?; // padding

        Ok(())
    }
}

#[derive(Clone, DdsData, Debug)]
pub struct SPDPdiscoveredParticipantData {
    pub domain_id: u16,
    pub _domain_tag: String,
    pub protocol_version: ProtocolVersion,
    #[key]
    pub guid: GUID, // Key
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

impl<C: speedy::Context> speedy::Writable<C> for SPDPdiscoveredParticipantData {
    #[inline]
    fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        // domain_id
        writer.write_u16(ParameterId::PID_DOMAIN_ID.value)?;
        writer.write_u16(4)?; // parameterLength
        writer.write_u32(self.domain_id as u32)?; // writing as u32 for proper alignment or matching CDR padding
                                                  // wait, earlier it wrote u16 and padded. We'll explicitly match original logic
                                                  // Original logic: domain_id as u16 + 2 byte padding = 4 bytes total
                                                  // But write_u32 handles it cleanly for Little Endian.

        // ProtocolVersion
        writer.write_u16(ParameterId::PID_PROTOCOL_VERSION.value)?;
        writer.write_u16(4)?;
        writer.write_value(&self.protocol_version.major)?;
        writer.write_value(&self.protocol_version.minor)?;
        writer.write_u16(0)?; // padding

        // VendorId
        writer.write_u16(ParameterId::PID_VENDOR_ID.value)?;
        writer.write_u16(4)?;
        writer.write_value(&self.vendor_id.vendor_id[0])?;
        writer.write_value(&self.vendor_id.vendor_id[1])?;
        writer.write_u16(0)?; // padding

        // expects_inline_qos
        writer.write_u16(ParameterId::PID_EXPECTS_INLINE_QOS.value)?;
        writer.write_u16(4)?;
        if self.expects_inline_qos {
            writer.write_u8(1)?;
        } else {
            writer.write_u8(0)?;
        }
        // padding alignment to 4 bytes for bool + u16
        writer.write_bytes(&[0; 3])?;

        // participant_guid
        writer.write_u16(ParameterId::PID_PARTICIPANT_GUID.value)?;
        writer.write_u16(16)?;
        writer.write_value(&self.guid)?;

        // metarraffic_unicast_locator_list
        for metarraffic_unicast_locator in &self.metarraffic_unicast_locator_list {
            writer.write_u16(ParameterId::PID_METATRAFFIC_UNICAST_LOCATOR.value)?;
            writer.write_u16(24)?;
            writer.write_value(metarraffic_unicast_locator)?;
        }

        // metarraffic_multicast_locator_list
        for metarraffic_multicast_locator in &self.metarraffic_multicast_locator_list {
            writer.write_u16(ParameterId::PID_METATRAFFIC_MULTICAST_LOCATOR.value)?;
            writer.write_u16(24)?;
            writer.write_value(metarraffic_multicast_locator)?;
        }

        // default_unicast_locator_list
        for default_unicast_locator in &self.default_unicast_locator_list {
            writer.write_u16(ParameterId::PID_DEFAULT_UNICAST_LOCATOR.value)?;
            writer.write_u16(24)?;
            writer.write_value(default_unicast_locator)?;
        }

        // default_multicast_locator_list
        for default_multicast_locator in &self.default_multicast_locator_list {
            writer.write_u16(ParameterId::PID_DEFAULT_MULTICAST_LOCATOR.value)?;
            writer.write_u16(24)?;
            writer.write_value(default_multicast_locator)?;
        }

        // available_builtin_endpoint
        writer.write_u16(ParameterId::PID_BUILTIN_ENDPOINT_SET.value)?;
        writer.write_u16(4)?;
        writer.write_i32(self.available_builtin_endpoint.bits() as i32)?;

        // lease_duration
        writer.write_u16(ParameterId::PID_PARTICIPANT_LEASE_DURATION.value)?;
        writer.write_u16(8)?;
        writer.write_value(&self.lease_duration)?;

        // manual_liveliness_count
        if let Some(manual_liveliness_count) = self.manual_liveliness_count {
            writer.write_u16(ParameterId::PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT.value)?;
            writer.write_u16(4)?;
            writer.write_value(&manual_liveliness_count)?;
        }

        // sentinel
        writer.write_u16(ParameterId::PID_SENTINEL.value)?;
        writer.write_u16(0)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::message::message_header::ProtocolVersion;
    use crate::message::submessage::element::{RepresentationIdentifier, SerializedPayload};
    use bytes::Bytes;
    use enumflags2::make_bitflags;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use speedy::{Readable, Writable};

    #[test]
    fn test_spdp_serialize() {
        let mut small_rng = SmallRng::seed_from_u64(0);

        let data = SPDPdiscoveredParticipantData::new(
            0,
            String::new(),
            ProtocolVersion::PROTOCOLVERSION,
            GUID::new_participant_guid(&mut small_rng),
            VendorId::THIS_IMPLEMENTATION,
            false,
            make_bitflags!(BuiltinEndpoint::{DISC_BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER}),
            vec![Locator::new_from_ipv4(7400, [127, 0, 0, 1])],
            vec![Locator::new_from_ipv4(7410, [127, 0, 0, 1])],
            vec![Locator::new_from_ipv4(7411, [127, 0, 0, 1])],
            vec![Locator::new_from_ipv4(7440, [127, 0, 0, 1])],
            Some(0),
            Duration {
                seconds: 20,
                fraction: 0,
            },
        );
        let serialized_payload =
            SerializedPayload::new_from_cdr_data(&data, RepresentationIdentifier::PL_CDR_LE);
        /*
        let mut serialized = String::new();
        let mut count = 0;
        for b in serialized_payload.value.clone() {
            serialized += &format!("{:>02X} ", b);
            count += 1;
            if count % 16 == 0 {
                serialized += "\n";
            } else if count % 8 == 0 {
                serialized += " ";
            }
        }
        println!("{}", serialized);
        */
        assert_eq!(
            Bytes::from_static(&[
                // 0x0F: PID_DOMAIN_ID, 0x15: PID_PROTOCOL_VERSION
                0x0F, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x15, 0x00, 0x04, 0x00, 0x02, 0x03,
                // 0x16: PID_VENDOR_ID, 0x43: PID_EXPECTS_INLINE_QOS
                0x00, 0x00, 0x16, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x43, 0x00, 0x04, 0x00,
                // 0x50: PID_PARTICIPANT_GUID,
                0x00, 0x00, 0x00, 0x00, 0x50, 0x00, 0x10, 0x00, 0x00, 0x00, 0xA1, 0x66, 0x86, 0xFF,
                // 0x32: PID_METATRAFFIC_UNICAST_LOCATOR
                0xC2, 0x11, 0x0A, 0x2C, 0x1D, 0x39, 0x00, 0x00, 0x01, 0xC1, 0x32, 0x00, 0x18, 0x00,
                0x01, 0x00, 0x00, 0x00, 0xE8, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x33: PID_METATRAFFIC_MULTICAST_LOCATOR
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7F, 0x00, 0x00, 0x01, 0x33, 0x00, 0x18, 0x00,
                0x01, 0x00, 0x00, 0x00, 0xF2, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x31: PID_DEFAULT_UNICAST_LOCATOR
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7F, 0x00, 0x00, 0x01, 0x31, 0x00, 0x18, 0x00,
                0x01, 0x00, 0x00, 0x00, 0xF3, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x48: PID_DEFAULT_MULTICAST_LOCATOR
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7F, 0x00, 0x00, 0x01, 0x48, 0x00, 0x18, 0x00,
                0x01, 0x00, 0x00, 0x00, 0x10, 0x1D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x48: PID_DEFAULT_MULTICAST_LOCATOR
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7F, 0x00, 0x00, 0x01, 0x58, 0x00, 0x04, 0x00,
                // 0x02: PID_PARTICIPANT_LEASE_DURATION
                0x3F, 0x0C, 0x00, 0x00, 0x02, 0x00, 0x08, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x34: PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, 0x01: PID_SENTINEL
                0x00, 0x00, 0x34, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            ]),
            serialized_payload.value
        );
    }

    #[test]
    fn test_spdp_deserialize() {
        let mut small_rng = SmallRng::seed_from_u64(0);

        let data = SPDPdiscoveredParticipantData::new(
            0,
            "hoge".to_string(),
            ProtocolVersion::PROTOCOLVERSION,
            GUID::new_participant_guid(&mut small_rng),
            VendorId::THIS_IMPLEMENTATION,
            false,
            make_bitflags!(BuiltinEndpoint::{DISC_BUILTIN_ENDPOINT_PARTICIPANT_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER|DISC_BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_WRITER|BUILTIN_ENDPOINT_PARTICIPANT_MESSAGE_DATA_READER}),
            vec![Locator::new_from_ipv4(7400, [127, 0, 0, 1])],
            vec![Locator::new_from_ipv4(7410, [127, 0, 0, 1])],
            vec![Locator::new_from_ipv4(7411, [127, 0, 0, 1])],
            vec![Locator::new_from_ipv4(7440, [127, 0, 0, 1])],
            Some(0),
            Duration {
                seconds: 20,
                fraction: 0,
            },
        );
        let serialized = data
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .expect("failed to serialize message");

        /*
        let mut serialized_msg = String::new();
        let mut count = 0;
        for b in serialized.clone() {
            serialized_msg += &format!("{:>02X} ", b);
            count += 1;
            if count % 16 == 0 {
                serialized_msg += "\n";
            } else if count % 8 == 0 {
                serialized_msg += " ";
            }
        }
        println!("{}", serialized_msg);
        */

        let mut deseriarized = match SDPBuiltinData::read_from_buffer_with_ctx(
            speedy::Endianness::LittleEndian,
            &serialized,
        ) {
            Ok(d) => d,
            Err(e) => panic!("failed to deserialize\n{}", e),
        };
        let new_data = deseriarized
            .gen_spdp_discoverd_participant_data()
            .expect("failed to get spdp data from SDPBuiltinData");
        assert_eq!(data.domain_id, new_data.domain_id);
        // assert_eq!(data._domain_tag, new_data._domain_tag);
        assert_eq!(data.protocol_version, new_data.protocol_version);
        assert_eq!(data.guid, new_data.guid);
        assert_eq!(data.vendor_id, new_data.vendor_id);
        assert_eq!(data.expects_inline_qos, new_data.expects_inline_qos);
        assert_eq!(
            data.available_builtin_endpoint,
            new_data.available_builtin_endpoint
        );
        assert_eq!(
            data.metarraffic_unicast_locator_list,
            new_data.metarraffic_unicast_locator_list
        );
        assert_eq!(
            data.metarraffic_multicast_locator_list,
            new_data.metarraffic_multicast_locator_list
        );
        assert_eq!(
            data.default_unicast_locator_list,
            new_data.default_unicast_locator_list
        );
        assert_eq!(
            data.default_multicast_locator_list,
            new_data.default_multicast_locator_list
        );
        assert_eq!(
            data.manual_liveliness_count,
            new_data.manual_liveliness_count
        );
        assert_eq!(data.lease_duration, new_data.lease_duration);
    }

    use crate::rtps::cache::HistoryCacheType;
    use crate::structure::{EntityId, EntityKind};
    #[test]
    fn test_sedp_w_serialize() {
        let writer_qos = DataWriterQosBuilder::new()
            .reliability(Reliability::default_besteffort())
            .build();
        let writer_proxy = WriterProxy::new(
            GUID {
                guid_prefix: GuidPrefix {
                    guid_prefix: [
                        0x00, 0x00, 0xa6, 0x0a, 0xb5, 0x76, 0xa5, 0x58, 0x15, 0xf3, 0xcc, 0x37,
                    ],
                },
                entity_id: EntityId::new(
                    [0x00, 0x03, 0x03],
                    EntityKind::WRITER_WITH_KEY_USER_DEFIND,
                ),
            },
            vec![Locator::new_from_ipv4(7411, [192, 168, 209, 2])],
            vec![Locator::new_from_ipv4(7401, [239, 255, 0, 1])],
            vec![],
            vec![],
            0,
            writer_qos,
            Arc::new(RwLock::new(HistoryCache::new(HistoryCacheType::Dummy))),
        );
        let publication_topic_data = PublicationBuiltinTopicData::new(
            None,
            None,
            Some(String::from("Square")),
            Some(String::from("ShapeType")),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let data = DiscoveredWriterData::new(writer_proxy, publication_topic_data);
        let serialized_payload =
            SerializedPayload::new_from_cdr_data(&data, RepresentationIdentifier::PL_CDR_LE);
        /*
        let mut serialized = String::new();
        let mut count = 0;
        for b in serialized_payload.value.clone() {
            serialized += &format!("{:>02x} ", b);
            count += 1;
            if count % 16 == 0 {
                serialized += "\n";
            } else if count % 8 == 0 {
                // serialized += " ";
            }
        }
        println!("{}", serialized);
        */
        assert_eq!(
            Bytes::from_static(&[
                // 0x5a: PID_ENDPOINT_GUID
                0x5a, 0x00, 0x10, 0x00, 0x00, 0x00, 0xa6, 0x0a, 0xb5, 0x76, 0xa5, 0x58, 0x15, 0xf3,
                // 0x2f: PID_UNICAST_LOCATOR
                0xcc, 0x37, 0x00, 0x03, 0x03, 0x02, 0x2f, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00,
                0xf3, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x30 PID_MULTICAST_LOCATOR
                0x00, 0x00, 0xc0, 0xa8, 0xd1, 0x02, 0x30, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00,
                0xe9, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x60: PID_TYPE_MAX_SIZE_SERIALIZED
                0x00, 0x00, 0xef, 0xff, 0x00, 0x01, 0x60, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x1d: PID_DURABILITY, 0x1e: PID_DURABILITY_SERVICE
                0x1d, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1e, 0x00, 0x1c, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                // 0x23: PID_DEADLINE
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x23, 0x00,
                // 0x27: PID_LATENCY_BUDGET
                0x08, 0x00, 0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff, 0xff, 0x27, 0x00, 0x08, 0x00,
                // 0x1b: PID_LIVELINESS
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1b, 0x00, 0x0c, 0x00, 0x00, 0x00,
                // 0x1a: PID_RELIABILITY
                0x00, 0x00, 0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff, 0xff, 0x1a, 0x00, 0x0c, 0x00,
                // 0x25: PID_DESTINATION_ORDER
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x99, 0x99, 0x99, 0x19, 0x25, 0x00,
                // 0x40: PID_HISTORY
                0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x41: PID_RESOURCE_LIMIT
                0x01, 0x00, 0x00, 0x00, 0x41, 0x00, 0x0c, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                // 0x49: PID_TRANSPORT_PRIORITY
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x49, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x2b: PID_LIFESPAN, 0x2c: PID_USER_DATA
                0x2b, 0x00, 0x08, 0x00, 0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff, 0xff, 0x2c, 0x00,
                // 0x1f: PID_OWNERSHIP
                0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x06: PID_OWNERSHIP_STRENGTH, 0x07: PID TYPE_NAME
                0x06, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00, 0x10, 0x00, 0x0a, 0x00,
                0x00, 0x00, 0x53, 0x68, 0x61, 0x70, 0x65, 0x54, 0x79, 0x70, 0x65, 0x00, 0x00, 0x00,
                // 0x05: PID_TOPIC_NAME
                0x05, 0x00, 0x0c, 0x00, 0x07, 0x00, 0x00, 0x00, 0x53, 0x71, 0x75, 0x61, 0x72, 0x65,
                // 0x01: PID_SENTINEL
                0x00, 0x00, 0x01, 0x00, 0x00, 0x00
            ]),
            serialized_payload.value
        );
    }
    #[test]
    fn test_sedp_w_deserialize() {
        let writer_qos = DataWriterQosBuilder::new()
            .reliability(Reliability::default_besteffort())
            .build();
        let guid = GUID {
            guid_prefix: GuidPrefix {
                guid_prefix: [
                    0x00, 0x00, 0xa6, 0x0a, 0xb5, 0x76, 0xa5, 0x58, 0x15, 0xf3, 0xcc, 0x37,
                ],
            },
            entity_id: EntityId::new([0x00, 0x03, 0x03], EntityKind::WRITER_WITH_KEY_USER_DEFIND),
        };
        let unicast_locator_list = vec![Locator::new_from_ipv4(7411, [192, 168, 209, 2])];
        let multicast_locator_list = vec![Locator::new_from_ipv4(7401, [239, 255, 0, 1])];
        let writer_proxy = WriterProxy::new(
            guid,
            unicast_locator_list.clone(),
            multicast_locator_list.clone(),
            vec![],
            vec![],
            0,
            writer_qos,
            Arc::new(RwLock::new(HistoryCache::new(HistoryCacheType::Dummy))),
        );
        let publication_topic_data = PublicationBuiltinTopicData::new(
            None,
            None,
            Some(String::from("Square")),
            Some(String::from("ShapeType")),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let data = DiscoveredWriterData::new(writer_proxy, publication_topic_data);
        let serialized = data
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .expect("failed to serialize message");
        /*
        let mut serialized = String::new();
        let mut count = 0;
        for b in serialized_payload.value.clone() {
            serialized += &format!("{:>02x} ", b);
            count += 1;
            if count % 16 == 0 {
                serialized += "\n";
            } else if count % 8 == 0 {
                // serialized += " ";
            }
        }
        println!("{}", serialized);
        */

        let mut deseriarized = match SDPBuiltinData::read_from_buffer_with_ctx(
            speedy::Endianness::LittleEndian,
            &serialized,
        ) {
            Ok(d) => d,
            Err(e) => panic!("failed to deserialize\n{}", e),
        };
        let writer_proxy = deseriarized
            .gen_writerproxy(
                Arc::new(RwLock::new(HistoryCache::new(HistoryCacheType::Dummy))),
                Vec::new(),
                Vec::new(),
            )
            .expect("failed generate writer_proxy from deserialized");
        //  TODO: compare qos
        assert_eq!(writer_proxy.remote_writer_guid, guid);
        assert_eq!(&writer_proxy.unicast_locator_list, &unicast_locator_list);
        assert_eq!(
            &writer_proxy.multicast_locator_list.clone(),
            &multicast_locator_list
        );
        assert_eq!(writer_proxy.data_max_size_serialized, 0);
        if let Some((topic_name, data_type)) = deseriarized.topic_info() {
            assert_eq!(topic_name, "Square");
            assert_eq!(data_type, "ShapeType");
        } else {
            panic!();
        };
    }
    #[test]
    fn test_sedp_r_serialize() {
        let reader_qos = DataReaderQosBuilder::new()
            .reliability(Reliability::default_besteffort())
            .build();
        let reader_proxy = ReaderProxy::new(
            GUID {
                guid_prefix: GuidPrefix {
                    guid_prefix: [
                        0x00, 0x00, 0xa6, 0x0a, 0xb5, 0x76, 0xa5, 0x58, 0x15, 0xf3, 0xcc, 0x37,
                    ],
                },
                entity_id: EntityId::new(
                    [0x00, 0x03, 0x03],
                    EntityKind::READER_WITH_KEY_USER_DEFIND,
                ),
            },
            false,
            vec![Locator::new_from_ipv4(7411, [192, 168, 209, 2])],
            vec![Locator::new_from_ipv4(7401, [239, 255, 0, 1])],
            vec![],
            vec![],
            reader_qos,
            Arc::new(RwLock::new(HistoryCache::new(HistoryCacheType::Dummy))),
            true,
        );
        let subscription_topic_data = SubscriptionBuiltinTopicData::new(
            None,
            None,
            Some(String::from("Square")),
            Some(String::from("ShapeType")),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let data = DiscoveredReaderData::new(reader_proxy, subscription_topic_data);
        let serialized_payload =
            SerializedPayload::new_from_cdr_data(&data, RepresentationIdentifier::PL_CDR_LE);
        /*
        let mut serialized = String::new();
        let mut count = 0;
        for b in serialized_payload.value.clone() {
            serialized += &format!("{:>02x} ", b);
            count += 1;
            if count % 16 == 0 {
                serialized += "\n";
            } else if count % 8 == 0 {
                // serialized += " ";
            }
        }
        println!("{}", serialized);
        */
        assert_eq!(
            Bytes::from_static(&[
                // 0x5a: PID_ENDPOINT_GUID
                0x5a, 0x00, 0x10, 0x00, 0x00, 0x00, 0xa6, 0x0a, 0xb5, 0x76, 0xa5, 0x58, 0x15, 0xf3,
                // 0x43: PID_EXPECTS_INLINE_QOS
                0xcc, 0x37, 0x00, 0x03, 0x03, 0x07, 0x43, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x2f: PID_UNICAST_LOCATOR
                0x2f, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00, 0xf3, 0x1c, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8, 0xd1, 0x02,
                // 0x30 PID_MULTICAST_LOCATOR
                0x30, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00, 0xe9, 0x1c, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xef, 0xff, 0x00, 0x01,
                // 0x1d: PID_DURABILITY, 0x23: PID_DEADLINE
                0x1d, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x23, 0x00, 0x08, 0x00, 0xff, 0xff,
                // 0x27: PID_LATENCY_BUDGET
                0xff, 0x7f, 0xff, 0xff, 0xff, 0xff, 0x27, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x1b: PID_LIVELINESS
                0x00, 0x00, 0x00, 0x00, 0x1b, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
                // 0x1a: PID_RELIABILITY
                0xff, 0x7f, 0xff, 0xff, 0xff, 0xff, 0x1a, 0x00, 0x0c, 0x00, 0x01, 0x00, 0x00, 0x00,
                // 0x25: PID_DESTINATION_ORDER
                0x00, 0x00, 0x00, 0x00, 0x99, 0x99, 0x99, 0x19, 0x25, 0x00, 0x04, 0x00, 0x00, 0x00,
                // 0x40: PID_HISTORY
                0x00, 0x00, 0x40, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                // 0x41: PID_RESOURCE_LIMIT
                0x41, 0x00, 0x0c, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                // 0x2c: PID_USER_DATA, 0x1f: PID_OWNERSHIP
                0xff, 0xff, 0x2c, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x04, 0x00,
                // 0x04: PID_TIME_BASED_FILTER
                0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                // 0x07: PID_TYPE_NAME
                0x00, 0x00, 0x07, 0x00, 0x10, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x53, 0x68, 0x61, 0x70,
                // 0x07: PID_TOPIC_NAME
                0x65, 0x54, 0x79, 0x70, 0x65, 0x00, 0x00, 0x00, 0x05, 0x00, 0x0c, 0x00, 0x07, 0x00,
                // 0x01: PID_SENTINEL
                0x00, 0x00, 0x53, 0x71, 0x75, 0x61, 0x72, 0x65, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00
            ]),
            serialized_payload.value
        );
    }
    #[test]
    fn test_sedp_r_deserialize() {
        let writer_qos = DataWriterQosBuilder::new()
            .reliability(Reliability::default_besteffort())
            .build();
        let guid = GUID {
            guid_prefix: GuidPrefix {
                guid_prefix: [
                    0x00, 0x00, 0xa6, 0x0a, 0xb5, 0x76, 0xa5, 0x58, 0x15, 0xf3, 0xcc, 0x37,
                ],
            },
            entity_id: EntityId::new([0x00, 0x03, 0x03], EntityKind::READER_WITH_KEY_USER_DEFIND),
        };
        let unicast_locator_list = vec![Locator::new_from_ipv4(7411, [192, 168, 209, 2])];
        let multicast_locator_list = vec![Locator::new_from_ipv4(7401, [239, 255, 0, 1])];
        let writer_proxy = WriterProxy::new(
            guid,
            unicast_locator_list.clone(),
            multicast_locator_list.clone(),
            vec![],
            vec![],
            0,
            writer_qos,
            Arc::new(RwLock::new(HistoryCache::new(HistoryCacheType::Dummy))),
        );
        let publication_topic_data = PublicationBuiltinTopicData::new(
            None,
            None,
            Some(String::from("Square")),
            Some(String::from("ShapeType")),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let data = DiscoveredWriterData::new(writer_proxy, publication_topic_data);
        let serialized = data
            .write_to_vec_with_ctx(speedy::Endianness::LittleEndian)
            .expect("failed to serialize message");
        /*
        let mut serialized = String::new();
        let mut count = 0;
        for b in serialized_payload.value.clone() {
            serialized += &format!("{:>02x} ", b);
            count += 1;
            if count % 16 == 0 {
                serialized += "\n";
            } else if count % 8 == 0 {
                // serialized += " ";
            }
        }
        println!("{}", serialized);
        */

        let mut deseriarized = match SDPBuiltinData::read_from_buffer_with_ctx(
            speedy::Endianness::LittleEndian,
            &serialized,
        ) {
            Ok(d) => d,
            Err(e) => panic!("failed to deserialize\n{}", e),
        };
        let writer_proxy = deseriarized
            .gen_writerproxy(
                Arc::new(RwLock::new(HistoryCache::new(HistoryCacheType::Dummy))),
                Vec::new(),
                Vec::new(),
            )
            .expect("failed generate writer_proxy from deserialized");
        //  TODO: compare qos
        assert_eq!(writer_proxy.remote_writer_guid, guid);
        assert_eq!(&writer_proxy.unicast_locator_list, &unicast_locator_list);
        assert_eq!(
            &writer_proxy.multicast_locator_list.clone(),
            &multicast_locator_list
        );
        assert_eq!(writer_proxy.data_max_size_serialized, 0);
        if let Some((topic_name, data_type)) = deseriarized.topic_info() {
            assert_eq!(topic_name, "Square");
            assert_eq!(data_type, "ShapeType");
        } else {
            panic!();
        };
    }
}
