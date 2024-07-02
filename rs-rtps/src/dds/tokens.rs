use crate::structure::entity_id::EntityId;
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
            let n: usize = token.try_into().unwrap();
            Self::Entity(EntityId::from_usize(n))
        }
    }
}

pub const PTB: usize = 0x40;
pub const STOP_POLL_TOKEN: Token = Token(PTB);
pub const ADD_WRITER_TOKEN: Token = Token(PTB + 1);
pub const REMOVE_WRITER_TOKEN: Token = Token(PTB + 2);
pub const ADD_READER_TOKEN: Token = Token(PTB + 3);
pub const REMOVE_READER_TOKEN: Token = Token(PTB + 4);
pub const DISCOVERY_UNI_TOKEN: Token = Token(PTB + 5);
pub const DISCOVERY_MULTI_TOKEN: Token = Token(PTB + 6);
pub const DISCOVERY_SEND_TOKEN: Token = Token(PTB + 7);
pub const SPDP_SEND_TIMER: Token = Token(PTB + 8);
pub const USERTRAFFIC_UNI_TOKEN: Token = Token(PTB + 9);
pub const USERTRAFFIC_MULTI_TOKEN: Token = Token(PTB + 10);
pub const DISCOVERY_DB_UPDATE: Token = Token(PTB + 11);
pub const SPDP_PARTICIPANT_DETECTOR: Token = Token(PTB + 12);
pub const SEDP_PUBLICATIONS_DETECTOR: Token = Token(PTB + 13);
pub const SEDP_SUBSCRIPTIONS_DETECTOR: Token = Token(PTB + 14);
pub const WRITER_HEARTBEAT_TIMER: Token = Token(PTB + 15);
pub const SET_READER_HEARTBEAT_TIMER: Token = Token(PTB + 16);
pub const READER_HEARTBEAT_TIMER: Token = Token(PTB + 17);
pub const DISC_WRITER_ADD: Token = Token(PTB + 19);
pub const DISC_READER_ADD: Token = Token(PTB + 20);
pub const SET_WRITER_NACK_TIMER: Token = Token(PTB + 21);
pub const WRITER_NACK_TIMER: Token = Token(PTB + 22);
