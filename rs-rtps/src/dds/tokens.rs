use mio::Token;

pub const PTB: usize = 0x40;
pub const STOP_POLL_TOKEN: Token = Token(PTB);
pub const ADD_WRITER_TOKEN: Token = Token(PTB + 1);
pub const REMOVE_WRITER_TOKEN: Token = Token(PTB + 2);
pub const ADD_READER_TOKEN: Token = Token(PTB + 3);
pub const REMOVE_READER_TOKEN: Token = Token(PTB + 4);
