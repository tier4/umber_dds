use crate::message::submessage::element::*;
use crate::message::submessage_flag::GapFlag;
use crate::structure::entity_id::*;
use enumflags2::BitFlags;
use speedy::{Readable, Writable};

#[derive(Readable, Writable, Clone)]
pub struct Gap {
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub gap_start: SequenceNumber,
    pub gap_list: SequenceNumberSet,
    pub gap_start_gsn: SequenceNumber,
    pub gap_end_gsn: SequenceNumber,
}

impl Gap {
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
            if self.gap_start_gsn <= SequenceNumber(0) {
                // gapStartGSN.value is zero or negative
                eprintln!("Invalid Gap Submessage received");
                return false;
            }
            if self.gap_end_gsn <= SequenceNumber(0) {
                // gapEndGSN.value is zero or negative
                eprintln!("Invalid Gap Submessage received");
                return false;
            }
            if self.gap_end_gsn < self.gap_start_gsn - SequenceNumber(1) {
                // gapEndGSN.value < gapStartGSN.value-1
                eprintln!("Invalid Gap Submessage received");
                return false;
            }
        }
        true
    }
}
