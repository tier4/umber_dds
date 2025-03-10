use crate::structure::TopicKind;
use crate::{Deserialize, Serialize};
use alloc::fmt;
use mio_v06::Token;
use speedy::{Readable, Writable};

// spec 9.2.2
#[derive(
    PartialEq, Readable, Writable, Clone, Copy, Eq, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct EntityId {
    entity_key: [u8; 3],
    entity_kind: EntityKind,
}

impl EntityId {
    pub fn new(entity_key: [u8; 3], entity_kind: EntityKind) -> Self {
        Self {
            entity_key,
            entity_kind,
        }
    }

    pub fn new_with_entity_kind(entity_key: [u8; 3], entity_kind: EntityKind) -> Self {
        Self {
            entity_key,
            entity_kind,
        }
    }

    pub fn as_token(&self) -> Token {
        let u = self.as_usize();
        Token(u)
    }

    pub fn is_reader(&self) -> bool {
        matches!(
            self.entity_kind,
            EntityKind::READER_WITH_KEY_BUILT_IN
                | EntityKind::READER_NO_KEY_BUILT_IN
                | EntityKind::READER_NO_KEY_USER_DEFIND
                | EntityKind::READER_WITH_KEY_USER_DEFIND
        )
    }

    pub fn is_writer(&self) -> bool {
        matches!(
            self.entity_kind,
            EntityKind::WRITER_WITH_KEY_BUILT_IN
                | EntityKind::WRITER_NO_KEY_BUILT_IN
                | EntityKind::WRITER_NO_KEY_USER_DEFIND
                | EntityKind::WRITER_WITH_KEY_USER_DEFIND
        )
    }

    pub fn topic_kind(&self) -> Option<TopicKind> {
        self.entity_kind.topic_kind()
    }

    pub fn entity_kind(&self) -> EntityKind {
        self.entity_kind
    }

    pub fn entity_key(&self) -> [u8; 3] {
        self.entity_key
    }

    pub const UNKNOW: Self = Self {
        entity_key: [0x00; 3],
        entity_kind: EntityKind::UNKNOW_USER_DEFIND,
    };

    // rtps spec: 9.3.1.3 Predefined EntityIds
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
        entity_kind: EntityKind::READER_WITH_KEY_BUILT_IN,
    };

    pub const SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: Self = Self {
        entity_key: [0x00, 0x00, 0x03],
        entity_kind: EntityKind::WRITER_WITH_KEY_BUILT_IN,
    };

    pub const SEDP_BUILTIN_PUBLICATIONS_DETECTOR: Self = Self {
        entity_key: [0x00, 0x00, 0x03],
        entity_kind: EntityKind::READER_WITH_KEY_BUILT_IN,
    };

    pub const SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: Self = Self {
        entity_key: [0x00, 0x00, 0x04],
        entity_kind: EntityKind::WRITER_WITH_KEY_BUILT_IN,
    };

    pub const SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: Self = Self {
        entity_key: [0x00, 0x00, 0x04],
        entity_kind: EntityKind::READER_WITH_KEY_BUILT_IN,
    };

    pub const SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: Self = Self {
        entity_key: [0x00, 0x01, 0x00],
        entity_kind: EntityKind::WRITER_WITH_KEY_BUILT_IN,
    };

    pub const SPDP_BUILTIN_PARTICIPANT_DETECTOR: Self = Self {
        entity_key: [0x00, 0x01, 0x00],
        entity_kind: EntityKind::READER_WITH_KEY_BUILT_IN,
    };

    pub const P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER: Self = Self {
        entity_key: [0x00, 0x02, 0x00],
        entity_kind: EntityKind::WRITER_WITH_KEY_BUILT_IN,
    };

    pub const P2P_BUILTIN_PARTICIPANT_MESSAGE_READER: Self = Self {
        entity_key: [0x00, 0x02, 0x00],
        entity_kind: EntityKind::READER_WITH_KEY_BUILT_IN,
    };

    fn as_usize(&self) -> usize {
        let u0 = self.entity_key[0] as u32;
        let u1 = self.entity_key[1] as u32;
        let u2 = self.entity_key[2] as u32;
        let u3 = self.entity_kind.value as u32;
        ((u0 << 24) | (u1 << 16) | (u2 << 8) | u3) as usize
    }

    pub fn from_usize(n: usize) -> Self {
        let ek = n & 0xff;
        let e2 = (n & 0xff00) >> 8;
        let e1 = (n & 0xff0000) >> 16;
        let e0 = (n & 0xff000000) >> 24;
        Self {
            entity_key: [e0 as u8, e1 as u8, e2 as u8],
            entity_kind: EntityKind { value: ek as u8 },
        }
    }
}

impl fmt::Debug for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UNKNOW => write!(f, "EntityId {{ UNKNOW }}"),
            Self::PARTICIPANT => write!(f, "EntityId {{ PARTICIPANT }}"),
            Self::SED_PBUILTIN_TOPICS_ANNOUNCER => {
                write!(f, "EntityId {{ SED_PBUILTIN_TOPICS_ANNOUNCER }}")
            }
            Self::SED_PBUILTIN_TOPICS_DETECTOR => {
                write!(f, "EntityId {{ SED_PBUILTIN_TOPICS_DETECTOR }}")
            }
            Self::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER => {
                write!(f, "EntityId {{ SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER }}")
            }
            Self::SEDP_BUILTIN_PUBLICATIONS_DETECTOR => {
                write!(f, "EntityId {{ SEDP_BUILTIN_PUBLICATIONS_DETECTOR }}")
            }
            Self::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER => {
                write!(f, "EntityId {{ SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER }}")
            }
            Self::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR => {
                write!(f, "EntityId {{ SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR }}")
            }
            Self::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER => {
                write!(f, "EntityId {{ SPDP_BUILTIN_PARTICIPANT_ANNOUNCER }}")
            }
            Self::SPDP_BUILTIN_PARTICIPANT_DETECTOR => {
                write!(f, "EntityId {{ SPDP_BUILTIN_PARTICIPANT_DETECTOR }}")
            }
            Self::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER => {
                write!(f, "EntityId {{ P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER }}")
            }
            Self::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER => {
                write!(f, "EntityId {{ P2P_BUILTIN_PARTICIPANT_MESSAGE_READER }}")
            }
            _ => write!(
                f,
                "EntityId {{ entity_key: {:?}, entity_kind: {:?} }}",
                self.entity_key, self.entity_kind
            ),
        }
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UNKNOW => write!(f, "EntityId {{ UNKNOW }}"),
            Self::PARTICIPANT => write!(f, "EntityId {{ PARTICIPANT }}"),
            Self::SED_PBUILTIN_TOPICS_ANNOUNCER => {
                write!(f, "EntityId {{ SED_PBUILTIN_TOPICS_ANNOUNCER }}")
            }
            Self::SED_PBUILTIN_TOPICS_DETECTOR => {
                write!(f, "EntityId {{ SED_PBUILTIN_TOPICS_DETECTOR }}")
            }
            Self::SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER => {
                write!(f, "EntityId {{ SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER }}")
            }
            Self::SEDP_BUILTIN_PUBLICATIONS_DETECTOR => {
                write!(f, "EntityId {{ SEDP_BUILTIN_PUBLICATIONS_DETECTOR }}")
            }
            Self::SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER => {
                write!(f, "EntityId {{ SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER }}")
            }
            Self::SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR => {
                write!(f, "EntityId {{ SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR }}")
            }
            Self::SPDP_BUILTIN_PARTICIPANT_ANNOUNCER => {
                write!(f, "EntityId {{ SPDP_BUILTIN_PARTICIPANT_ANNOUNCER }}")
            }
            Self::SPDP_BUILTIN_PARTICIPANT_DETECTOR => {
                write!(f, "EntityId {{ SPDP_BUILTIN_PARTICIPANT_DETECTOR }}")
            }
            Self::P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER => {
                write!(f, "EntityId {{ P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITER }}")
            }
            Self::P2P_BUILTIN_PARTICIPANT_MESSAGE_READER => {
                write!(f, "EntityId {{ P2P_BUILTIN_PARTICIPANT_MESSAGE_READER }}")
            }
            _ => write!(
                f,
                "EntityId {{ entity_key: {:?}, entity_kind: {:?} }}",
                self.entity_key, self.entity_kind
            ),
        }
    }
}

#[derive(
    PartialEq, Readable, Writable, Clone, Copy, Eq, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct EntityKind {
    value: u8,
}

impl EntityKind {
    // spce 9.3.1.2
    pub const UNKNOW_USER_DEFIND: Self = Self { value: 0x00 };
    pub const WRITER_WITH_KEY_USER_DEFIND: Self = Self { value: 0x02 };
    pub const WRITER_NO_KEY_USER_DEFIND: Self = Self { value: 0x03 };
    pub const READER_NO_KEY_USER_DEFIND: Self = Self { value: 0x04 };
    pub const READER_WITH_KEY_USER_DEFIND: Self = Self { value: 0x07 };
    pub const WRITER_GROUP_USER_DEFIND: Self = Self { value: 0x08 };
    pub const READER_GROUP_USER_DEFIND: Self = Self { value: 0x09 };

    pub const UNKNOW_BUILT_IN: Self = Self { value: 0xc0 };
    pub const PARTICIPANT_BUILT_IN: Self = Self { value: 0xc1 };
    pub const WRITER_WITH_KEY_BUILT_IN: Self = Self { value: 0xc2 };
    pub const WRITER_NO_KEY_BUILT_IN: Self = Self { value: 0xc3 };
    pub const READER_NO_KEY_BUILT_IN: Self = Self { value: 0xc4 };
    pub const READER_WITH_KEY_BUILT_IN: Self = Self { value: 0xc7 };
    pub const WRITER_GROUP_BUILT_IN: Self = Self { value: 0xc8 };
    pub const READER_GROUP_BUILT_IN: Self = Self { value: 0xc9 };

    // self difined
    pub const PUBLISHER: Self = Self { value: 0x40 };
    pub const SUBSCRIBER: Self = Self { value: 0x41 };
}

impl EntityKind {
    fn topic_kind(&self) -> Option<TopicKind> {
        match *self {
            Self::WRITER_WITH_KEY_USER_DEFIND
            | Self::WRITER_WITH_KEY_BUILT_IN
            | Self::READER_WITH_KEY_USER_DEFIND
            | Self::READER_WITH_KEY_BUILT_IN => Some(TopicKind::WithKey),
            Self::WRITER_NO_KEY_USER_DEFIND
            | Self::WRITER_NO_KEY_BUILT_IN
            | Self::READER_NO_KEY_USER_DEFIND
            | Self::READER_NO_KEY_BUILT_IN => Some(TopicKind::NoKey),
            _ => None,
        }
    }
}

impl fmt::Debug for EntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UNKNOW_USER_DEFIND => write!(f, "EntityKind: UNKNOW_USER_DEFIND(0x00)"),
            Self::WRITER_WITH_KEY_USER_DEFIND => {
                write!(f, "EntityKind: WRITER_WITH_KEY_USER_DEFIND(0x02)")
            }
            Self::WRITER_NO_KEY_USER_DEFIND => {
                write!(f, "EntityKind: WRITER_NO_KEY_USER_DEFIND(0x03)")
            }
            Self::READER_NO_KEY_USER_DEFIND => {
                write!(f, "EntityKind: READER_NO_KEY_USER_DEFIND(0x04)")
            }
            Self::READER_WITH_KEY_USER_DEFIND => {
                write!(f, "EntityKind: READER_WITH_KEY_USER_DEFIND(0x07)")
            }
            Self::WRITER_GROUP_USER_DEFIND => {
                write!(f, "EntityKind: WRITER_GROUP_USER_DEFIND(0x08)")
            }
            Self::READER_GROUP_USER_DEFIND => {
                write!(f, "EntityKind: READER_GROUP_USER_DEFIND(0x09)")
            }
            Self::UNKNOW_BUILT_IN => write!(f, "EntityKind: UNKNOW_BUILT_IN(0xC0)"),
            Self::PARTICIPANT_BUILT_IN => write!(f, "EntityKind: PARTICIPANT_BUILT_IN(0xC1)"),
            Self::WRITER_WITH_KEY_BUILT_IN => {
                write!(f, "EntityKind: WRITER_WITH_KEY_BUILT_IN(0xC2)")
            }
            Self::WRITER_NO_KEY_BUILT_IN => write!(f, "EntityKind: WRITER_NO_KEY_BUILT_IN(0xC3)"),
            Self::READER_NO_KEY_BUILT_IN => write!(f, "EntityKind: READER_NO_KEY_BUILT_IN(0xC4)"),
            Self::READER_WITH_KEY_BUILT_IN => {
                write!(f, "EntityKind: READER_WITH_KEY_BUILT_IN(0xC7)")
            }
            Self::WRITER_GROUP_BUILT_IN => write!(f, "EntityKind: WRITER_GROUP_BUILT_IN(0xC8)"),
            Self::READER_GROUP_BUILT_IN => write!(f, "EntityKind: READER_GROUP_BUILT_IN(0xC9)"),
            Self::PUBLISHER => write!(f, "EntityKind: PUBLISHER(0x40)"),
            Self::SUBSCRIBER => write!(f, "EntityKind: SUBSCRIBER(0x41)"),
            _ => write!(f, "EntityKind: OTHER(0x{:02X})", self.value),
        }
    }
}
