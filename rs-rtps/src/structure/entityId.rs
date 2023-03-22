// spec 9.2.2
pub struct EntityId {
    entityKey: [u8; 3],
    entityKind: EntityKind,
}

impl EntityId {
    pub const UNKNOW: Self = Self {
        entityKey: [0x00; 3],
        entityKind: EntityKind::UNKNOW_USER_DEFIND,
    };
}

struct EntityKind {
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
}
