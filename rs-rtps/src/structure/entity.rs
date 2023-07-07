use crate::structure::{
    entity_id::EntityId,
    guid::{GuidPrefix, GUID},
};

pub trait RTPSEntity {
    fn guid(&self) -> GUID;
    fn entity_id(&self) -> EntityId {
        self.guid().entity_id
    }
    fn guid_prefix(&self) -> GuidPrefix {
        self.guid().guid_prefix
    }
}
