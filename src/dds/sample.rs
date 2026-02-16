use crate::message::submessage::element::Timestamp;
use crate::DdsData;
use serde::Deserialize;

pub struct DataSample<D: for<'de> Deserialize<'de> + DdsData> {
    data: D,
    sample_info: SampleInfo,
}

impl<D: for<'de> Deserialize<'de> + DdsData> DataSample<D> {
    pub(crate) fn new(data: D, sample_info: SampleInfo) -> Self {
        Self { data, sample_info }
    }

    pub fn data(&self) -> &D {
        &self.data
    }

    pub fn sample_info(&self) -> &SampleInfo {
        &self.sample_info
    }
}

#[derive(Debug)]
pub struct SampleInfo {
    source_timestamp: Timestamp,
}

impl SampleInfo {
    pub(crate) fn new(source_ts: Timestamp) -> Self {
        Self {
            source_timestamp: source_ts,
        }
    }
}
