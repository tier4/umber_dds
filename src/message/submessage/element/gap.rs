use crate::message::submessage::element::*;
use crate::message::submessage_flag::GapFlag;
use crate::structure::EntityId;
use enumflags2::BitFlags;
use speedy::{Readable, Writable};

#[derive(Readable, Clone)]
pub struct Gap {
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub gap_start: SequenceNumber,
    pub gap_list: SequenceNumberSet,
    #[speedy(default_on_eof)]
    pub gap_start_gsn: Option<SequenceNumber>,
    #[speedy(default_on_eof)]
    pub gap_end_gsn: Option<SequenceNumber>,
}

impl Gap {
    pub fn new(
        reader_id: EntityId,
        writer_id: EntityId,
        gap_start: SequenceNumber,
        gap_list: SequenceNumberSet,
    ) -> Self {
        Self {
            reader_id,
            writer_id,
            gap_start,
            gap_list,
            gap_start_gsn: None,
            gap_end_gsn: None,
        }
    }
    pub fn is_valid(&self, flag: BitFlags<GapFlag>) -> bool {
        // rtps 2.3 spec 8.3.7.4 Gap
        // validation
        if self.gap_start <= SequenceNumber(0) {
            // gapStart is zero or negative
            eprintln!("Invalid Gap Submessage received");
            return false;
        }
        if !self.gap_list.is_valid() {
            // gapList is invalid
            eprintln!("Invalid Gap Submessage received");
            return false;
        }
        if flag.contains(GapFlag::GroupInfo) {
            // GroupInfoFlag is set and
            match self.gap_start_gsn {
                Some(gs_gsn) => {
                    if gs_gsn <= SequenceNumber(0) {
                        // gapStartGSN.value is zero or negative
                        eprintln!("Invalid Gap Submessage received");
                        return false;
                    }
                }
                None => {
                    eprintln!("Invalid Gap Submessage received");
                    return false;
                }
            }
            match self.gap_end_gsn {
                Some(ge_gsn) => {
                    if ge_gsn <= SequenceNumber(0) {
                        // gapEndGSN.value is zero or negative
                        eprintln!("Invalid Gap Submessage received");
                        return false;
                    }
                    if ge_gsn < self.gap_start_gsn.unwrap() - SequenceNumber(1) {
                        // gapEndGSN.value < gapStartGSN.value-1
                        eprintln!("Invalid Gap Submessage received");
                        return false;
                    }
                }
                None => {
                    eprintln!("Invalid Gap Submessage received");
                    return false;
                }
            }
        }
        true
    }
}

impl<C: Context> Writable<C> for Gap {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_value(&self.reader_id)?;
        writer.write_value(&self.writer_id)?;
        writer.write_value(&self.gap_start)?;
        writer.write_value(&self.gap_list)?;
        if let Some(gs_gsn) = self.gap_start_gsn {
            writer.write_value(&gs_gsn)?
        }
        if let Some(ge_gsn) = self.gap_end_gsn {
            writer.write_value(&ge_gsn)?
        }
        Ok(())
    }
}
