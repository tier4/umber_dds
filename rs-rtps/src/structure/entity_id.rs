use crate::dds::participant::DomainParticipant;
use speedy::Readable;

// spec 9.2.2
#[derive(PartialEq, Readable, Clone, Copy)]
pub struct EntityId {
    entity_key: [u8; 3],
    entity_kind: EntityKind,
}

impl EntityId {
    pub fn new_with_entity_kind(dp: &DomainParticipant, entity_kind: EntityKind) -> Self {
        Self {
            entity_key: dp.gen_entity_key(),
            entity_kind,
        }
    }

    pub const UNKNOW: Self = Self {
        entity_key: [0x00; 3],
        entity_kind: EntityKind::UNKNOW_USER_DEFIND,
    };

    pub const PARTICIPANT: Self = Self {
        entity_key: [0x00, 0x00, 0x01],
        entity_kind: EntityKind::PARTICIPANT_BUILT_IN,
    };

    pub const SED_PBUILTIN_TOPICS_ANNOUNCER: Self = Self {
        entity_key: [0x00, 0x00, 0x02],
        entity_kind: EntityKind::WRITER_WITH_KEY_BUILT_IN,
    };

    pub const SED_PBUILTIN_TOPICS_DETECTOR: Self = Self {
        entity_key: [0x00, 0x00, 0x02],
        entity_kind: EntityKind::READER_NO_KEY_BUILT_IN,
    };

    pub const SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: Self = Self {
        entity_key: [0x00, 0x00, 0x03],
        entity_kind: EntityKind::READER_NO_KEY_BUILT_IN,
    };

    pub const SEDP_BUILTIN_PUBLICATIONS_DETECTOR: Self = Self {
        entity_key: [0x00, 0x00, 0x03],
        entity_kind: EntityKind::READER_NO_KEY_BUILT_IN,
    };

    pub const SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: Self = Self {
        entity_key: [0x00, 0x00, 0x04],
        entity_kind: EntityKind::WRITER_WITH_KEY_BUILT_IN,
    };

    pub const SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: Self = Self {
        entity_key: [0x00, 0x00, 0x04],
        entity_kind: EntityKind::READER_NO_KEY_BUILT_IN,
    };

    pub const SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: Self = Self {
        entity_key: [0x00, 0x01, 0x00],
        entity_kind: EntityKind::WRITER_WITH_KEY_BUILT_IN,
    };

    pub const SPDP_BUILTIN_PARTICIPANT_DETECTOR: Self = Self {
        entity_key: [0x00, 0x01, 0x00],
        entity_kind: EntityKind::READER_NO_KEY_BUILT_IN,
    };

    pub const P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER: Self = Self {
        entity_key: [0x00, 0x02, 0x00],
        entity_kind: EntityKind::WRITER_WITH_KEY_BUILT_IN,
    };

    pub const P2P_BUILTIN_PARTICIPANT_MESSAGE_READER: Self = Self {
        entity_key: [0x00, 0x02, 0x00],
        entity_kind: EntityKind::READER_NO_KEY_BUILT_IN,
    };
}

#[derive(PartialEq, Readable, Clone, Copy)]
pub struct EntityKind {
    value: u8,
}

impl EntityKind {
    // spce 9.3.1.2
    pub const UNKNOW_USER_DEFIND: Self = Self { value: 0x00 };
    pub const WRITER_WITH_KEY_USER_DEFIND: Self = Self { value: 0x02 };
    pub const WRITER_NO_KEY_USER_DEFIND: Self = Self { value: 0x03 };
    pub const READER_WITH_KEY_USER_DEFIND: Self = Self { value: 0x04 };
    pub const READER_NO_KEY_USER_DEFIND: Self = Self { value: 0x07 };
    pub const WRITER_GROUP_USER_DEFIND: Self = Self { value: 0x08 };
    pub const READER_GROUP_USER_DEFIND: Self = Self { value: 0x09 };

    pub const UNKNOW_BUILT_IN: Self = Self { value: 0x00 };
    pub const PARTICIPANT_BUILT_IN: Self = Self { value: 0xc1 };
    pub const WRITER_WITH_KEY_BUILT_IN: Self = Self { value: 0xc2 };
    pub const WRITER_NO_KEY_BUILT_IN: Self = Self { value: 0xc3 };
    pub const READER_WITH_KEY_BUILT_IN: Self = Self { value: 0xc4 };
    pub const READER_NO_KEY_BUILT_IN: Self = Self { value: 0xc7 };
    pub const WRITER_GROUP_BUILT_IN: Self = Self { value: 0xc8 };
    pub const READER_GROUP_BUILT_IN: Self = Self { value: 0xc9 };

    // self difined
    pub const PUBLISHER: Self = Self { value: 0x40 };
    pub const SUBSCRIBER: Self = Self { value: 0x41 };
}
