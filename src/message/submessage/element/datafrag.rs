use crate::error::{IoError, IoResult};
use crate::message::submessage::{element::*, submessage_flag::DataFragFlag};
use crate::structure::EntityId;
use bytes::Bytes;
use enumflags2::BitFlags;
use speedy::{Endianness, Error, Readable};

pub struct DataFrag {
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub fragment_starting_num: FragmentNumber,
    pub fragments_in_submessage: u16,
    pub data_size: u32,
    pub fragment_size: u16,
    pub inline_qos: Option<ParameterList>,
    /// Note: RustDDS says
    /// "RTPS spec says the serialized_payload is of type SerializedPayload
    /// but that is technically incorrect. It is a fragment of
    /// SerializedPayload. The headers at the beginning of SerializedPayload
    /// appear only at the first fragment. The fragmentation mechanism here
    /// should treat serialized_payload as an opaque stream of bytes."
    pub serialized_payload: SerializedPayload,
}

impl DataFrag {
    pub fn deserialize(buffer: &Bytes, flags: BitFlags<DataFragFlag>) -> IoResult<Self> {
        let mut readed_byte = 0;
        let endiannes = if flags.contains(DataFragFlag::Endianness) {
            Endianness::LittleEndian
        } else {
            Endianness::BigEndian
        };
        let map_speedy_err = |p: Error| IoError::SpeedyError(p);

        let _extra_flags = u16::read_from_buffer_with_ctx(endiannes, &buffer.slice(readed_byte..))
            .map_err(map_speedy_err)?;
        readed_byte += 2;
        let octets_to_inline_qos =
            u16::read_from_buffer_with_ctx(endiannes, &buffer.slice(readed_byte..))
                .map_err(map_speedy_err)?;
        readed_byte += 2;
        let reader_id =
            EntityId::read_from_buffer_with_ctx(endiannes, &buffer.slice(readed_byte..))
                .map_err(map_speedy_err)?;
        readed_byte += 4;
        let writer_id =
            EntityId::read_from_buffer_with_ctx(endiannes, &buffer.slice(readed_byte..))
                .map_err(map_speedy_err)?;
        readed_byte += 4;
        let writer_sn =
            SequenceNumber::read_from_buffer_with_ctx(endiannes, &buffer.slice(readed_byte..))
                .map_err(map_speedy_err)?;
        readed_byte += 8;
        let fragment_starting_num =
            FragmentNumber::read_from_buffer_with_ctx(endiannes, &buffer.slice(readed_byte..))
                .map_err(map_speedy_err)?;
        readed_byte += 4;
        let fragments_in_submessage =
            u16::read_from_buffer_with_ctx(endiannes, &buffer.slice(readed_byte..))
                .map_err(map_speedy_err)?;
        readed_byte += 2;
        let fragment_size = u16::read_from_buffer_with_ctx(endiannes, &buffer.slice(readed_byte..))
            .map_err(map_speedy_err)?;
        readed_byte += 2;
        let data_size = u32::read_from_buffer_with_ctx(endiannes, &buffer.slice(readed_byte..))
            .map_err(map_speedy_err)?;
        readed_byte += 4;
        let is_exist_inline_qos = flags.contains(DataFragFlag::InlineQos);

        // between octets_to_inline_qos and inline_qos in rtps 2.3, there are
        // reader_id (4), writer_id (4), writer_sn (8), fragment_staring_num (4), fragment_in_submessage
        // (2), fragment_size (2), data_size (4) = 28 octets
        let rtps_v23_datafrag_header_size: u16 = 28;
        let extra_octets = octets_to_inline_qos - rtps_v23_datafrag_header_size;
        readed_byte += u64::from(extra_octets) as usize;

        let inline_qos = if is_exist_inline_qos {
            let param_list =
                ParameterList::read_from_buffer_with_ctx(endiannes, &buffer.slice(readed_byte..))
                    .map_err(map_speedy_err)?;
            let param_list_byte = param_list
                .write_to_vec_with_ctx(endiannes)
                .map_err(map_speedy_err)?;
            readed_byte += param_list_byte.len();
            Some(param_list)
        } else {
            None
        };
        // TODO: Validity checks
        //
        let serialized_payload =
            SerializedPayload::from_bytes(&buffer.clone().split_off(readed_byte)).unwrap();

        Ok(Self {
            reader_id,
            writer_id,
            writer_sn,
            fragment_starting_num,
            fragments_in_submessage,
            data_size,
            fragment_size,
            inline_qos,
            serialized_payload,
        })
    }
}

impl<C: Context> Writable<C> for DataFrag {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u16(0)?;
        if self.inline_qos.is_some() && !self.inline_qos.as_ref().unwrap().parameters.is_empty() {
            todo!()
        } else if self.inline_qos.is_some()
            && self.inline_qos.as_ref().unwrap().parameters.is_empty()
            || self.inline_qos.is_none()
        {
            writer.write_u16(24)?;
        }
        writer.write_value(&self.reader_id)?;
        writer.write_value(&self.writer_id)?;
        writer.write_value(&self.writer_sn)?;
        writer.write_value(&self.fragment_starting_num)?;
        writer.write_value(&self.fragments_in_submessage)?;
        writer.write_value(&self.fragment_size)?;
        writer.write_value(&self.data_size)?;
        if self.inline_qos.is_some() && !self.inline_qos.as_ref().unwrap().parameters.is_empty() {
            writer.write_value(&self.inline_qos)?;
        }
        writer.write_value(&self.serialized_payload)?;
        Ok(())
    }
}
