use crate::message::submessage::element::*;
use crate::structure::entity_id::*;
use speedy::{Readable, Writable};

#[derive(Readable)]
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

impl<C: Context> Writable<C> for Heartbeat {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_value(&self.reader_id)?;
        writer.write_value(&self.writer_id)?;
        writer.write_value(&self.first_sn)?;
        writer.write_value(&self.last_sn)?;
        writer.write_value(&self.count)?;
        match self.current_gsn {
            Some(cgsn) => writer.write_value(&cgsn)?,
            None => (),
        }
        match self.first_gsn {
            Some(fgsn) => writer.write_value(&fgsn)?,
            None => (),
        }
        match self.last_gsn {
            Some(lgsn) => writer.write_value(&lgsn)?,
            None => (),
        }
        match self.writer_set {
            Some(ws) => writer.write_value(&ws)?,
            None => (),
        }
        match self.secure_writer_set {
            Some(sws) => writer.write_value(&sws)?,
            None => (),
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
