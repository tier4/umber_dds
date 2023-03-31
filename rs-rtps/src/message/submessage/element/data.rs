use crate::message::submessage::{element::*, submessage_flag::DataFlag};
use crate::structure::entityId::*;
use enumflags2::BitFlags;
use speedy::{Context, Endianness, Error, Readable};
use std::io;

pub struct Data {
    readerId: EntityId,
    writerId: EntityId,
    writerSN: SequenceNumber,
    inlineQos: Option<ParameterList>,
    serializedPayload: Option<SerializedPayload>,
}

impl Data {
    pub fn deserialize_data(buffer: &Bytes, flags: BitFlags<DataFlag>) -> std::io::Result<Self> {
        let mut cursor = io::Cursor::new(&buffer);
        let endiannes = if flags.contains(DataFlag::Endianness) {
            Endianness::LittleEndian
        } else {
            Endianness::BigEndian
        };
        let map_speedy_err = |p: Error| io::Error::new(io::ErrorKind::Other, p);

        let _extra_flags = u16::read_from_stream_unbuffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let octets_to_inline_qos =
            u16::read_from_stream_unbuffered_with_ctx(endiannes, &mut cursor)
                .map_err(map_speedy_err)?;
        let readerId = EntityId::read_from_stream_unbuffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let writerId = EntityId::read_from_stream_unbuffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let writerSN = SequenceNumber::read_from_stream_unbuffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let is_exist_inline_qos = flags.contains(DataFlag::InlineQos);
        let is_exist_serialized_data =
            flags.contains(DataFlag::Datqa) || flags.contains(DataFlag::Key);

        // between octets_to_inline_qos and inlineQos in rtps n2.3, there are
        // readerId (4), writerId (4), writerSN (8) = 16 octets
        let rtps_v23_data_header_size: u16 = 16;
        let extra_octes = octets_to_inline_qos - rtps_v23_data_header_size;
        cursor.set_position(cursor.position() + u64::from(extra_octes));

        let inlineQos = if is_exist_inline_qos {
            // TODO: bug fix
            // 以下のsubmessageを受け取るとParameterList::read_from_stream_unbuffered_with_ctx()で"failed to fill whole buffer"というエラーが発生する。
            // submessage header: SubMessageHeader { submessageId: 21, flags: 3, submessageLength: 80 }
            /*
            submessage DATA: 0x00 0x00 0x10 0x00 0x00 0x01 0x00 0xC7 0x00 0x01 0x00 0xC2 0x00 0x00 0x00 0x00 0x02 0x00 0x00 0x00 0x0F 0x80 0x18 0x00 0x01 0x0F 0x19 0x1A 0x44 0x01 0x3E 0x4D 0x01 0x00 0x00 0x00 0x00 0x01 0x00 0xC2 0x00 0x00 0x00 0x00 0x01 0x00 0x00 0x00 0x70 0x00 0x10 0x00 0x01 0x0F 0x19 0x1A 0x44 0x01 0x3E 0x4D 0x01 0x00 0x00 0x00 0x00 0x00 0x01 0xC1 0x71 0x00 0x04 0x00 0x00 0x00 0x00 0x03 0x01 0x00 0x00 0x00
            */
            Some(
                ParameterList::read_from_stream_unbuffered_with_ctx(endiannes, &mut cursor)
                    .map_err(map_speedy_err)?,
            )
        } else {
            None
        };

        let serializedPayload = if is_exist_serialized_data {
            Some(SerializedPayload::from_bytes(
                &buffer.clone().split_off(cursor.position() as usize),
            )?)
        } else {
            None
        };
        Ok(Self {
            readerId,
            writerId,
            writerSN,
            inlineQos,
            serializedPayload,
        })
    }
}
