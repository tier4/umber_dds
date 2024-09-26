use crate::message::submessage::element::*;
use crate::message::submessage_flag::HeartbeatFlag;
use crate::structure::EntityId;
use enumflags2::BitFlags;
use speedy::{Readable, Writable};

#[derive(Readable, Clone)]
pub struct Heartbeat {
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub first_sn: SequenceNumber,
    pub last_sn: SequenceNumber,
    pub count: Count,
    // Present only if the GroupInfoFlag is set in the header.
    #[speedy(default_on_eof)]
    pub current_gsn: Option<SequenceNumber>,
    // Present only if the GroupInfoFlag is set in the header.
    #[speedy(default_on_eof)]
    pub first_gsn: Option<SequenceNumber>,
    // Present only if the GroupInfoFlag is set in the header.
    #[speedy(default_on_eof)]
    pub last_gsn: Option<SequenceNumber>,
    // Present only if the GroupInfoFlag is set in the header.
    #[speedy(default_on_eof)]
    pub writer_set: Option<GroupDigest>,
    // Present only if the GroupInfoFlag is set in the header.
    #[speedy(default_on_eof)]
    pub secure_writer_set: Option<GroupDigest>,
}

impl Heartbeat {
    #![allow(clippy::too_many_arguments)]
    pub fn new(
        reader_id: EntityId,
        writer_id: EntityId,
        first_sn: SequenceNumber,
        last_sn: SequenceNumber,
        count: Count,
        current_gsn: Option<SequenceNumber>,
        first_gsn: Option<SequenceNumber>,
        last_gsn: Option<SequenceNumber>,
        writer_set: Option<GroupDigest>,
        secure_writer_set: Option<GroupDigest>,
    ) -> Self {
        Self {
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
            current_gsn,
            first_gsn,
            last_gsn,
            writer_set,
            secure_writer_set,
        }
    }

    pub fn is_valid(&self, flag: BitFlags<HeartbeatFlag>) -> bool {
        // rtps 2.3 spec 8.3.7.5 Heartbeat
        if self.first_sn <= SequenceNumber(0) {
            // first_sn is zero or negative
            eprintln!("Invalid Heartbeat Submessage received");
            return false;
        }
        if self.last_sn < SequenceNumber(0) {
            // last_sn is negative
            eprintln!("Invalid Heartbeat Submessage received");
            return false;
        }
        if self.last_sn < self.first_sn - SequenceNumber(1) {
            // lastSN.value < firstSN.value - 1
            eprintln!("Invalid Heartbeat Submessage received");
            return false;
        }
        if flag.contains(HeartbeatFlag::GroupInfo) {
            // TODO: GrupuInfo falgが立っていれば、必ず*_gsnがNoneでないかどうか確認
            if self.current_gsn.unwrap() <= SequenceNumber(0) {
                // currentGNS is zero or negative
                eprintln!("Invalid Heartbeat Submessage received");
                return false;
            }
            if self.first_gsn.unwrap() <= SequenceNumber(0) {
                // firstGSN is zero or negative
                eprintln!("Invalid Heartbeat Submessage received");
                return false;
            }
            if self.last_gsn.unwrap() < SequenceNumber(0) {
                // lastGSN is negative
                eprintln!("Invalid Heartbeat Submessage received");
                return false;
            }
            if self.last_gsn.unwrap() < self.first_gsn.unwrap() - SequenceNumber(1) {
                // lastGSN.value < firstGSN.value - 1
                eprintln!("Invalid Heartbeat Submessage received");
                return false;
            }
            if self.current_gsn.unwrap() < self.first_gsn.unwrap() {
                // currentGNS.value < firstGSN.value
                eprintln!("Invalid Heartbeat Submessage received");
                return false;
            }
            if self.current_gsn.unwrap() < self.last_gsn.unwrap() {
                // currentGNS.value < lastGSN.value
                eprintln!("Invalid Heartbeat Submessage received");
                return false;
            }
        }
        true
    }
}

impl<C: Context> Writable<C> for Heartbeat {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_value(&self.reader_id)?;
        writer.write_value(&self.writer_id)?;
        writer.write_value(&self.first_sn)?;
        writer.write_value(&self.last_sn)?;
        writer.write_value(&self.count)?;
        if let Some(cgsn) = self.current_gsn {
            writer.write_value(&cgsn)?
        }
        if let Some(fgsn) = self.first_gsn {
            writer.write_value(&fgsn)?
        }
        if let Some(lgsn) = self.last_gsn {
            writer.write_value(&lgsn)?
        }
        if let Some(ws) = self.writer_set {
            writer.write_value(&ws)?
        }
        if let Some(sws) = self.secure_writer_set {
            writer.write_value(&sws)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::message::submessage::element::heartbeat::Heartbeat;
    use bytes::Bytes;
    use speedy::{Endianness, Readable};

    #[test]
    fn test_deserialize() {
        const TEST_HEARTBEAT: [u8; 28] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0xC2, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
        ];
        const E: Endianness = Endianness::LittleEndian;
        let test_heartbeat = Bytes::from_static(&TEST_HEARTBEAT);
        match Heartbeat::read_from_buffer_with_ctx(E, &test_heartbeat) {
            Ok(_) => (),
            Err(e) => panic!("{:?}", e),
        };
    }
}
