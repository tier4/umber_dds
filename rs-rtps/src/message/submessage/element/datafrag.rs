use crate::message::submessage::{element::*, submessage_flag::DataFragFlag};
use crate::structure::entity_id::*;
use bytes::Bytes;
use enumflags2::BitFlags;
use speedy::{Endianness, Error, Readable};
use std::io;

pub struct DataFrag {
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    fragment_staring_num: FragmentNumber,
    fragment_in_submessage: u16,
    data_size: u64,
    fragment_size: u16,
    inline_qos: Option<ParameterList>,
    /// Note: RustDDS says
    /// "RTPS spec says the serialized_payload is of type SerializedPayload
    /// but that is technically incorrect. It is a fragment of
    /// SerializedPayload. The headers at the beginning of SerializedPayload
    /// appear only at the first fragment. The fragmentation mechanism here
    /// should treat serialized_payload as an opaque stream of bytes."
    serialized_payload: Bytes,
}

impl DataFrag {
    pub fn deserialize(buffer: &Bytes, flags: BitFlags<DataFragFlag>) -> io::Result<Self> {
        let mut cursor = io::Cursor::new(&buffer);
        let endiannes = if flags.contains(DataFragFlag::Endianness) {
            Endianness::LittleEndian
        } else {
            Endianness::BigEndian
        };
        let map_speedy_err = |p: Error| io::Error::new(io::ErrorKind::Other, p);

        let _extra_flags = u16::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let octets_to_inline_qos = u16::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let reader_id = EntityId::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let writer_id = EntityId::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let writer_sn = SequenceNumber::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let fragment_staring_num =
            FragmentNumber::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
                .map_err(map_speedy_err)?;
        let fragment_in_submessage = u16::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let data_size = u64::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let fragment_size = u16::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let is_exist_inline_qos = flags.contains(DataFragFlag::InlineQos);

        // between octets_to_inline_qos and inline_qos in rtps n2.3, there are
        // reader_id (4), writer_id (4), writer_sn (8), fragment_staring_num (4), fragment_in_submessage
        // (2), data_size (8), fragment_size (2) = 32 octets
        let rtps_v23_datafrag_header_size: u16 = 32;
        let extra_octets = octets_to_inline_qos - rtps_v23_datafrag_header_size;
        cursor.set_position(cursor.position() + u64::from(extra_octets));

        let inline_qos = if is_exist_inline_qos {
            Some(
                ParameterList::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
                    .map_err(map_speedy_err)?,
            )
        } else {
            None
        };
        // TODO: Validity checks
        //
        let serialized_payload = buffer.clone().split_off(cursor.position() as usize);

        Ok(Self {
            reader_id,
            writer_id,
            writer_sn,
            fragment_staring_num,
            fragment_in_submessage,
            data_size,
            fragment_size,
            inline_qos,
            serialized_payload,
        })
    }
}
