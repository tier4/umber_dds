use crate::message::submessage::{element::*, submessage_flag::DataFlag};
use crate::structure::entity_id::*;
use enumflags2::BitFlags;
use speedy::{Context, Endianness, Error, Readable};
use std::io;

pub struct Data {
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    inline_qos: Option<ParameterList>,
    serialized_payload: Option<SerializedPayload>,
}

impl Data {
    pub fn new(
        reader_id: EntityId,
        writer_id: EntityId,
        writer_sn: SequenceNumber,
        inline_qos: Option<ParameterList>,
        serialized_payload: Option<SerializedPayload>,
    ) -> Self {
        Self {
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        }
    }

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
        let reader_id = EntityId::read_from_stream_unbuffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let writer_id = EntityId::read_from_stream_unbuffered_with_ctx(endiannes, &mut cursor)
            .map_err(map_speedy_err)?;
        let writer_sn =
            SequenceNumber::read_from_stream_unbuffered_with_ctx(endiannes, &mut cursor)
                .map_err(map_speedy_err)?;
        let is_exist_inline_qos = flags.contains(DataFlag::InlineQos);
        let is_exist_serialized_data =
            flags.contains(DataFlag::Datqa) || flags.contains(DataFlag::Key);

        // between octets_to_inline_qos and inline_qos in rtps n2.3, there are
        // reader_id (4), writer_id (4), writer_sn (8) = 16 octets
        let rtps_v23_data_header_size: u16 = 16;
        let extra_octes = octets_to_inline_qos - rtps_v23_data_header_size;
        cursor.set_position(cursor.position() + u64::from(extra_octes));

        let inline_qos = if is_exist_inline_qos {
            Some(
                ParameterList::read_from_stream_unbuffered_with_ctx(endiannes, &mut cursor)
                    .map_err(map_speedy_err)?,
            )
        } else {
            None
        };

        let serialized_payload = if is_exist_serialized_data {
            Some(SerializedPayload::from_bytes(
                &buffer.clone().split_off(cursor.position() as usize),
            )?)
        } else {
            None
        };
        Ok(Self {
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::message::submessage::submessage_flag::DataFlag;
    use bytes::Bytes;
    use enumflags2::BitFlags;

    #[test]
    fn test_sezialize() {
        const TEST_DATA: [u8; 80] = [
            0x00, 0x00, 0x10, 0x00, 0x00, 0x01, 0x00, 0xC7, 0x00, 0x01, 0x00, 0xC2, 0x00, 0x00,
            0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0F, 0x80, 0x18, 0x00, 0x01, 0x0F, 0x19, 0x1A,
            0x44, 0x01, 0x3E, 0x4D, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0xC2, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x70, 0x00, 0x10, 0x00, 0x01, 0x0F, 0x19, 0x1A,
            0x44, 0x01, 0x3E, 0x4D, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xC1, 0x71, 0x00,
            0x04, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x00, 0x00, 0x00,
        ];
        let test_data_flags: BitFlags<DataFlag, u8> =
            BitFlags::<DataFlag>::from_bits_truncate(0x03);
        let data = Bytes::from_static(&TEST_DATA);
        match Data::deserialize_data(&data, test_data_flags) {
            Ok(_) => (),
            Err(e) => panic!("{:?}", e),
        }
    }
}
