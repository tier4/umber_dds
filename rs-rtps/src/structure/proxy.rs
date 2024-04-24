use crate::message::submessage::element::Locator;
use crate::structure::{guid::GUID, parameter_id::ParameterId};
use serde::{ser::SerializeStruct, Serialize};

#[derive(Clone)]
pub struct ReaderProxy {
    pub remote_reader_guid: GUID,
    expects_inline_qos: bool,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
}

impl ReaderProxy {
    pub fn new(
        remote_reader_guid: GUID,
        expects_inline_qos: bool,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
    ) -> Self {
        Self {
            remote_reader_guid,
            expects_inline_qos,
            unicast_locator_list,
            multicast_locator_list,
        }
    }
}
impl Serialize for ReaderProxy {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ReaderProxy", 4)?;
        // remote_reader_guid
        s.serialize_field("parameterId", &ParameterId::PID_ENDPOINT_GUID.value)?;
        s.serialize_field::<u16>("parameterLength", &16)?;
        s.serialize_field("remote_reader_guid", &self.remote_reader_guid)?;

        // expects_inline_qos
        s.serialize_field("parameterId", &ParameterId::PID_EXPECTS_INLINE_QOS.value)?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("expects_inline_qos", &self.expects_inline_qos)?;
        s.serialize_field::<u16>("padding", &0)?;

        // unicast_locator_list
        for unicast_locator in &self.unicast_locator_list {
            s.serialize_field("parameterId", &ParameterId::PID_UNICAST_LOCATOR.value)?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field("unicast_locator_list", &unicast_locator)?;
        }

        // multicast_locator_list
        for multicast_locator in &self.multicast_locator_list {
            s.serialize_field("parameterId", &ParameterId::PID_MULTICAST_LOCATOR.value)?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field("multicast_locator_list", &multicast_locator)?;
        }

        s.end()
    }
}

#[derive(Clone)]
pub struct WriterProxy {
    remote_writer_guid: GUID,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    data_max_size_serialized: i32, // in rtps 2.3 spec, Figure 8.30: long
}

impl WriterProxy {
    pub fn new(
        remote_writer_guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: i32,
    ) -> Self {
        Self {
            remote_writer_guid,
            unicast_locator_list,
            multicast_locator_list,
            data_max_size_serialized,
        }
    }
}
impl Serialize for WriterProxy {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("WriterProxy", 4)?;
        // remote_writer_guid
        s.serialize_field("parameterId", &ParameterId::PID_ENDPOINT_GUID.value)?;
        s.serialize_field::<u16>("parameterLength", &16)?;
        s.serialize_field("remote_reader_guid", &self.remote_writer_guid)?;

        // unicast_locator_list
        for unicast_locator in &self.unicast_locator_list {
            s.serialize_field("parameterId", &ParameterId::PID_UNICAST_LOCATOR.value)?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field("unicast_locator_list", &unicast_locator)?;
        }

        // multicast_locator_list
        for multicast_locator in &self.multicast_locator_list {
            s.serialize_field("parameterId", &ParameterId::PID_MULTICAST_LOCATOR.value)?;
            s.serialize_field::<u16>("parameterLength", &24)?;
            s.serialize_field("multicast_locator_list", &multicast_locator)?;
        }

        // data_max_size_serialized
        s.serialize_field(
            "parameterId",
            &ParameterId::PID_TYPE_MAX_SIZE_SERIALIZED.value,
        )?;
        s.serialize_field::<u16>("parameterLength", &4)?;
        s.serialize_field("data_max_size_serialized", &self.data_max_size_serialized)?;

        s.end()
    }
}
