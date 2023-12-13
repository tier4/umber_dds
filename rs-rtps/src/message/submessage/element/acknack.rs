use crate::message::submessage::element::*;
use crate::structure::entity_id::*;
use speedy::{Readable, Writable};

#[derive(Readable, Writable)]
pub struct AckNack {
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub reader_sn_state: SequenceNumberSet,
    pub count: Count,
}

#[cfg(test)]
mod test {
    use crate::message::submessage::element::acknack::AckNack;
    use bytes::Bytes;
    use speedy::{Endianness, Readable};

    #[test]
    fn test_deserialize() {
        const TEST_ACKNACK1: [u8; 28] = [
            0x00, 0x00, 0x03, 0xc7, 0x00, 0x00, 0x03, 0xc2, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x03, 0x00, 0x00, 0x00,
        ];
        const TEST_ACKNACK2: [u8; 24] = [
            0x00, 0x00, 0x04, 0xc7, 0x00, 0x00, 0x04, 0xc2, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
        ];
        const E: Endianness = Endianness::LittleEndian;
        let test_acknack1 = Bytes::from_static(&TEST_ACKNACK1);
        let test_acknack2 = Bytes::from_static(&TEST_ACKNACK2);
        match AckNack::read_from_buffer_with_ctx(E, &test_acknack1) {
            Ok(ack) => (),
            Err(e) => panic!("{:?}", e),
        };
        match AckNack::read_from_buffer_with_ctx(E, &test_acknack2) {
            Ok(ack) => (),
            Err(e) => panic!("{:?}", e),
        };
    }
}
