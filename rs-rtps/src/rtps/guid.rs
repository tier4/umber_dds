use rand;
use speedy::Readable;

pub struct GUID {
    guidPrefix: GuidPrefix,
    entityId: EntityId,
}

impl GUID {
    pub const UNKNOW: Self = Self {guidPrefix: GuidPrefix::UNKNOW, entityId: EntityId::UNKNOW};
}

#[derive(Readable, Debug)]
pub struct GuidPrefix {
    pub guid_prefix: [u8; 12],
}
impl GuidPrefix {
    pub const UNKNOW: Self = Self {guid_prefix: [0x00; 12]};

    // sprc 9.3.1.5
    // guidPrefix[0] = venderId[0]
    // guidPrefix[1] = venderId[1]
    pub fn new() -> Self {
        let mut bytes: [u8; 12] = rand::random();
        bytes[0] = crate::rtps::vendorId::VendorId::THIS_IMPLEMENTATION.vendor_id[0];
        bytes[1] = crate::rtps::vendorId::VendorId::THIS_IMPLEMENTATION.vendor_id[1];
        Self {guid_prefix: bytes}
    }
}

// spec 9.2.2
pub struct EntityId {
    entityKey: [u8; 3],
    entityKind: EntityKind,
}

impl EntityId {
    pub const UNKNOW: Self = Self { entityKey: [0x00; 3], entityKind: EntityKind::UNKNOW_USER_DEFIND };
}

pub struct EntityKind {
    value: u8,
}

impl EntityKind {
    // spce 9.3.1.2
    pub const UNKNOW_USER_DEFIND: Self = Self{value: 0x00};
    pub const WRITER_WITH_KEY_USER_DEFIND: Self = Self{value: 0x02};
    pub const WRITER_NO_KEY_USER_DEFIND: Self = Self{value: 0x03};
    pub const READER_WITH_KEY_USER_DEFIND: Self = Self{value: 0x04};
    pub const READER_NO_KEY_USER_DEFIND: Self = Self{value: 0x07};
    pub const WRITER_GROUP_USER_DEFIND: Self = Self{value: 0x08};
    pub const READER_GROUP_USER_DEFIND: Self = Self{value: 0x09};


    pub const UNKNOW_BUILT_IN: Self = Self{value: 0x00};
    pub const PARTICIPANT_BUILT_IN: Self = Self{value: 0xc1};
    pub const WRITER_WITH_KEY_BUILT_IN: Self = Self{value: 0xc2};
    pub const WRITER_NO_KEY_BUILT_IN: Self = Self{value: 0xc3};
    pub const READER_WITH_KEY_BUILT_IN: Self = Self{value: 0xc4};
    pub const READER_NO_KEY_BUILT_IN: Self = Self{value: 0xc7};
    pub const WRITER_GROUP_BUILT_IN: Self = Self{value: 0xc8};
    pub const READER_GROUP_BUILT_IN: Self = Self{value: 0xc9};

}
