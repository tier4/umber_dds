use crate::message::submessage::{element::*, submessage_flag::DataFragFlag};
use crate::structure::entityId::*;
use bytes::Bytes;
use enumflags2::BitFlags;
use speedy::{Endianness, Error, Readable};
use std::io;

pub struct DataFrag {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    fragmentStaringNum: FragmentNumber,
    fragmentInSubmessage: u16,
    dataSize: u64,
    fragmentSize: u16,
    inlineQos: Option<ParameterList>,
    /// Note: RustDDS says
    /// "RTPS spec says the serialized_payload is of type SerializedPayload
    /// but that is technically incorrect. It is a fragment of
    /// SerializedPayload. The headers at the beginning of SerializedPayload
    /// appear only at the first fragment. The fragmentation mechanism here
    /// should treat serialized_payload as an opaque stream of bytes."
    serializedPayload: Bytes,
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
        let readerId = EntityId::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let writerId = EntityId::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let writerSN = SequenceNumber::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let fragmentStaringNum =
            FragmentNumber::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
                .map_err(map_speedy_err)?;
        let fragmentInSubmessage = u16::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let dataSize = u64::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let fragmentSize = u16::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let is_exist_inline_qos = flags.contains(DataFragFlag::InlineQos);

        // between octets_to_inline_qos and inlineQos in rtps n2.3, there are
        // readerId (4), writerId (4), writerSN (8), fragmentStaringNum (4), fragmentInSubmessage
        // (2), dataSize (8), fragmentSize (2) = 32 octets
        let rtps_v23_datafrag_header_size: u16 = 32;
        let extra_octets = octets_to_inline_qos - rtps_v23_datafrag_header_size;
        cursor.set_position(cursor.position() + u64::from(extra_octets));

        let inlineQos = if is_exist_inline_qos {
            Some(
                ParameterList::read_from_stream_buffered_with_ctx(endiannes, &mut cursor)
                    .map_err(map_speedy_err)?,
            )
        } else {
            None
        };
        // TODO: Validity checks
        //
        let serializedPayload = buffer.clone().split_off(cursor.position() as usize);

        Ok(Self {
            readerId,
            writerId,
            writerSN,
            fragmentStaringNum,
            fragmentInSubmessage,
            dataSize,
            fragmentSize,
            inlineQos,
            serializedPayload,
        })
    }
}
