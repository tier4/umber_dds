use crate::structure::EntityId;
use mio_v06::Token;

pub enum TokenDec {
    ReservedToken(Token),
    Entity(EntityId),
}

impl TokenDec {
    pub fn decode(token: Token) -> Self {
        if Token(0x40) <= token && Token(0x60) > token {
            Self::ReservedToken(token)
        } else {
            let n: usize = token.into();
            Self::Entity(EntityId::from_usize(n))
        }
    }
}

#[allow(dead_code)]
pub const PTB: usize = 0x40;
pub const _STOP_POLL_TOKEN: Token = Token(PTB);
pub const ADD_WRITER_TOKEN: Token = Token(PTB + 0x1);
pub const _REMOVE_WRITER_TOKEN: Token = Token(PTB + 0x2);
pub const ADD_READER_TOKEN: Token = Token(PTB + 0x3);
pub const _REMOVE_READER_TOKEN: Token = Token(PTB + 0x4);
pub const DISCOVERY_UNI_TOKEN: Token = Token(PTB + 0x5);
pub const DISCOVERY_MULTI_TOKEN: Token = Token(PTB + 0x6);
pub const DISCOVERY_SEND_TOKEN: Token = Token(PTB + 0x7);
pub const SPDP_SEND_TIMER: Token = Token(PTB + 0x8);
pub const USERTRAFFIC_UNI_TOKEN: Token = Token(PTB + 0x9);
pub const USERTRAFFIC_MULTI_TOKEN: Token = Token(PTB + 0xA);
pub const DISCOVERY_DB_UPDATE: Token = Token(PTB + 0xB);
pub const SPDP_PARTICIPANT_DETECTOR: Token = Token(PTB + 0xC);
pub const _SEDP_PUBLICATIONS_DETECTOR: Token = Token(PTB + 0xD);
pub const _SEDP_SUBSCRIPTIONS_DETECTOR: Token = Token(PTB + 0xE);
pub const WRITER_HEARTBEAT_TIMER: Token = Token(PTB + 0xF);
pub const SET_READER_HEARTBEAT_TIMER: Token = Token(PTB + 0x10);
pub const READER_HEARTBEAT_TIMER: Token = Token(PTB + 0x11);
pub const DISC_WRITER_ADD: Token = Token(PTB + 0x12);
pub const DISC_READER_ADD: Token = Token(PTB + 0x13);
pub const SET_WRITER_NACK_TIMER: Token = Token(PTB + 0x14);
pub const WRITER_NACK_TIMER: Token = Token(PTB + 0x15);
pub const PARTICIPANT_MESSAGE_READER: Token = Token(PTB + 0x16);
pub const WRITER_LIVELINESS_CHECK_TIMER: Token = Token(PTB + 0x17);
pub const ASSERT_LIVELINESS_TIMER: Token = Token(PTB + 0x18);
pub const SET_WLP_TIMER: Token = Token(PTB + 0x19);
pub const PARTICIPANT_LIVELINESS_TIMER: Token = Token(PTB + 0x1A);
