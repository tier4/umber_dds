use enumflags2::{bitflags, BitFlags};
use speedy::Endianness;

macro_rules! from_enndianness {
    ($flag_type: ident) => {
        impl $flag_type {
            #[allow(dead_code)]
            pub fn from_enndianness(endiann: Endianness) -> BitFlags<$flag_type> {
                if endiann == Endianness::LittleEndian {
                    $flag_type::Endianness.into()
                } else {
                    BitFlags::<$flag_type>::empty()
                }
            }
        }
    };
}

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum AckNackFlag {
    Endianness = 0b01,
    Final = 0b10,
}
from_enndianness!(AckNackFlag);

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum DataFlag {
    Endianness = 0b00001,
    InlineQos = 0b00010,
    Data = 0b00100,
    Key = 0b01000,
    NonStandardPayload = 0b10000,
}
from_enndianness!(DataFlag);

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum DataFragFlag {
    Endianness = 0b0001,
    InlineQos = 0b0010,
    NonStandardPayload = 0b0100,
    Key = 0b1000,
}
from_enndianness!(DataFragFlag);

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum GapFlag {
    Endianness = 0b01,
    GroupInfo = 0b10,
}
from_enndianness!(GapFlag);

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum HeartbeatFlag {
    Endianness = 0b0001,
    Final = 0b0010,
    Liveliness = 0b0100,
    GroupInfo = 0b1000,
}
from_enndianness!(HeartbeatFlag);

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum HeartbeatFragFlag {
    Endianness = 0b1,
}
from_enndianness!(HeartbeatFragFlag);

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum InfoDestionationFlag {
    Endianness = 0b1,
}
from_enndianness!(InfoDestionationFlag);

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum InfoReplyFlag {
    Endianness = 0b01,
    Multicast = 0b10,
}
from_enndianness!(InfoReplyFlag);

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum InfoReplyIp4Flag {
    Endianness = 0b01,
    Multicast = 0b10,
}
from_enndianness!(InfoReplyIp4Flag);

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum InfoSourceFlag {
    Endianness = 0b1,
}
from_enndianness!(InfoSourceFlag);

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum InfoTimestampFlag {
    Endianness = 0b01,
    Invalidate = 0b10,
}
from_enndianness!(InfoTimestampFlag);

#[derive(Clone, Copy)]
#[bitflags]
#[repr(u8)]
pub enum NackFragFlag {
    Endianness = 0b1,
}
from_enndianness!(NackFragFlag);
