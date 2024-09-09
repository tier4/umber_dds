//! structures for DDS & RTPS

mod duration;
mod entity;
mod entity_id;
mod guid;
mod parameter_id;
mod proxy;
mod topic_kind;
mod vendor_id;

#[doc(inline)]
pub use {
    duration::Duration,
    entity::RTPSEntity,
    entity_id::{EntityId, EntityKind},
    guid::{GuidPrefix, GUID},
    topic_kind::TopicKind,
};

pub(crate) use {
    parameter_id::ParameterId,
    proxy::{ReaderProxy, WriterProxy},
    vendor_id::VendorId,
};
