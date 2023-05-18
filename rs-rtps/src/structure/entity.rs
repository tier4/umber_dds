use crate::structure::{
    entityId::EntityId,
    guid::{GuidPrefix, GUID},
};

pub trait RTPSEntity {
    fn guid(&self) -> GUID;
    fn entity_id(&self) -> EntityId {
        self.guid().entityId
    }
    fn guid_prefix(&self) -> GuidPrefix {
        self.guid().guidPrefix
    }
}
