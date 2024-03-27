pub mod acknack;
pub mod data;
pub mod datafrag;
pub mod gap;
pub mod heartbeat;
pub mod heartbeatfrag;
pub mod infodst;
pub mod inforeply;
pub mod inforeply_ip4;
pub mod infosrc;
pub mod infots;
pub mod nackfrag;

use crate::network::net_util::get_local_interfaces;
use crate::structure::parameter_id::ParameterId;
use byteorder::ReadBytesExt;
use bytes::{BufMut, Bytes, BytesMut};
use cdr::{CdrBe, CdrLe, Infinite, PlCdrBe, PlCdrLe};
use chrono::Local;
use serde::{Deserialize, Serialize};
use speedy::{Context, Readable, Reader, Writable, Writer};
use std::io;
use std::net::IpAddr;
use std::ops::{Add, AddAssign};

// spec 9.4.2 Mapping of the PIM SubmessageElements

pub type Count = i32;

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy)]
pub struct SequenceNumber(pub i64); // The precise definition of SequenceNumber is:
                                    // struct SequenceNumber {high: i32, low: u32}.
                                    // Therefore, when serialized in LittleEndian, SequenceNumber(0) is represented as:
                                    // [0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00].
                                    // However, this accurate representation has lower performance
                                    // for addition and subtraction operations. Hence, we use a faster implementation
                                    // for these operations and, during serialization or deserialization,
                                    // we convert to the correct format. This is the reason we
                                    // implement the serializer and deserializer manually.
impl SequenceNumber {
    pub const SEQUENCENUMBER_UNKNOWN: Self = Self((std::u32::MAX as i64) << 32);
    pub const MAX: Self = Self(i64::MAX);
    pub const MIN: Self = Self(i64::MIN);
}
impl Add for SequenceNumber {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}
impl AddAssign for SequenceNumber {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<'a, C: Context> Readable<'a, C> for SequenceNumber {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let high: i32 = reader.read_value()?;
        let low: u32 = reader.read_value()?;

        Ok(SequenceNumber(((i64::from(high)) << 32) + i64::from(low)))
    }
}

impl<C: Context> Writable<C> for SequenceNumber {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_i32((self.0 >> 32) as i32)?;
        writer.write_u32(self.0 as u32)?;
        Ok(())
    }
}

pub type FragmentNumber = u32;

pub type SequenceNumberSet = NumberSet<SequenceNumber>;
impl SequenceNumberSet {
    pub fn validate(&self) -> bool {
        // rtps spec 9.4.2.6 SequenceNumberSet
        let mut is_valid = true;
        if self.bitmap_base < SequenceNumber(1) {
            is_valid = false;
        }
        if self.num_bits > 256 {
            is_valid = false
        }
        if self.bitmap.len() as u32 == (self.num_bits + 31) / 32 {
            is_valid = false
        }
        is_valid
    }
}
pub type FragmentNumberSet = NumberSet<FragmentNumber>;

#[derive(Readable, Writable)]
pub struct NumberSet<T> {
    pub bitmap_base: T,
    pub num_bits: u32,
    #[speedy(length = (num_bits + 31) / 32)]
    pub bitmap: Vec<u32>,
}

#[derive(PartialEq)]
pub struct Parameter {
    parameter_id: ParameterId,
    // length: i16,
    // RTPS 2.3 spec 9.4.2.11 ParameterList show Parameter contains length,
    // but it need only deseriarize time
    value: Vec<u8>,
}
impl<C: Context> Writable<C> for Parameter {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_value(&self.parameter_id)?;

        let length = self.value.len();
        let alignment = length % 4;
        writer.write_u16((length + alignment) as u16)?;

        for byte in &self.value {
            writer.write_u8(*byte)?;
        }

        for _ in 0..alignment {
            writer.write_u8(0x00)?;
        }

        Ok(())
    }
}

#[derive(Default, PartialEq)]
pub struct ParameterList {
    parameters: Vec<Parameter>,
}

impl<'a, C: Context> Readable<'a, C> for ParameterList {
    fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let mut parameter_list = ParameterList::default();
        loop {
            let parameter_id = ParameterId::read_from(reader)?;
            match parameter_id {
                ParameterId::PID_SENTINEL => return Ok(parameter_list),
                _ => {
                    let length = u16::read_from(reader)?;
                    let value = reader.read_vec(length as usize)?;
                    parameter_list.parameters.push(Parameter {
                        parameter_id,
                        value,
                    })
                }
            }
        }
    }
}

const SENTINEL: u32 = 0x00000001;
impl<C: Context> Writable<C> for ParameterList {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        for param in self.parameters.iter() {
            writer.write_value(param)?;
        }

        writer.write_u32(SENTINEL)?;

        Ok(())
    }
}

#[derive(Readable, Writable)]
pub struct Timestamp {
    pub seconds: u32,
    pub fraction: u32,
}

impl Timestamp {
    pub const TIME_INVALID: Self = Self {
        seconds: 0xFFFFFFFF,
        fraction: 0xFFFFFFFF,
    };
    pub const TIME_ZERO: Self = Self {
        seconds: 0x00,
        fraction: 0x00,
    };
    pub const TIME_INFINITE: Self = Self {
        seconds: 0xFFFFFFFF,
        fraction: 0xFFFFFFFE,
    };

    pub fn now() -> Option<Self> {
        let now = Local::now().timestamp_nanos_opt()?;
        Some(Self {
            seconds: (now / 1_000_000_000) as u32,
            fraction: (now % 1_000_000_000) as u32,
        })
    }
}

// spec versin 2.3, 9.3.2 Mapping of the Types that Appear Within Submessages or Built-in Topic Data
#[derive(Readable, Writable, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Locator {
    kind: i32,
    port: u32,
    // spec version 2.3, 9.3.2.3 Locator_t
    // if address contains an IPv4 address. In this case, the leading 12 octets of the
    //  address must be zero. The last 4 octets are used to store the IPv4 address.
    address: [u8; 16],
}

impl Locator {
    pub const KIND_INVALID: i32 = -1;
    pub const KIND_RESERVED: i32 = 0;
    pub const KIND_UDPV4: i32 = 1;
    pub const KIND_UDPV6: i32 = 2;
    pub const PORT_INVALID: u32 = 0;
    pub const ADDRESS_INVALID: [u8; 16] = [0; 16];
    pub const INVALID: Self = Self {
        kind: Self::KIND_INVALID,
        port: Self::PORT_INVALID,
        address: Self::ADDRESS_INVALID,
    };

    pub fn new(kind: i32, port: u32, address: [u8; 16]) -> Self {
        Locator {
            kind,
            port,
            address,
        }
    }

    pub fn new_list_from_self_ipv4(port: u32) -> Vec<Self> {
        let addrs = get_local_interfaces();
        let mut address = [0; 4];
        let mut locators = Vec::new();
        for a in addrs {
            match a {
                IpAddr::V4(v4a) => {
                    address = v4a.octets();
                    println!("self ipv4: {:?}", address);
                    assert_ne!([0; 4], address);

                    let mut addr: [u8; 16] = [0; 16];
                    addr[..12].copy_from_slice(&[0; 12]);
                    addr[12..].copy_from_slice(&address);
                    locators.push(Locator {
                        kind: Self::KIND_UDPV4,
                        port,
                        address: addr,
                    });
                }
                IpAddr::V6(_v6a) => (),
            }
        }
        locators
    }

    pub fn new_from_ipv4(port: u32, address: [u8; 4]) -> Self {
        let mut addr: [u8; 16] = [0; 16];
        addr[..12].copy_from_slice(&[0; 12]);
        addr[12..].copy_from_slice(&address);
        Locator {
            kind: Self::KIND_UDPV4,
            port,
            address: addr,
        }
    }
}

pub type LocatorList = Vec<Locator>;

#[derive(Readable, Writable, Clone, Copy, PartialEq, Eq)]
pub struct RepresentationIdentifier {
    bytes: [u8; 2],
}

impl RepresentationIdentifier {
    pub fn bytes(&self) -> [u8; 2] {
        self.bytes
    }
    // Numeric values are from RTPS spec v2.3 Section 10.5 , Table 10.3
    pub const CDR_BE: Self = Self {
        bytes: [0x00, 0x00],
    };
    pub const CDR_LE: Self = Self {
        bytes: [0x00, 0x01],
    };
    pub const PL_CDR_BE: Self = Self {
        bytes: [0x00, 0x02],
    };
    pub const PL_CDR_LE: Self = Self {
        bytes: [0x00, 0x03],
    };
    pub const CDR2_BE: Self = Self {
        bytes: [0x00, 0x10],
    };
    pub const CDR2_LE: Self = Self {
        bytes: [0x00, 0x11],
    };
    pub const PL_CDR2_BE: Self = Self {
        bytes: [0x00, 0x12],
    };
    pub const PL_CDR2_LE: Self = Self {
        bytes: [0x00, 0x13],
    };
    pub const D_CDR_BE: Self = Self {
        bytes: [0x00, 0x14],
    };
    pub const D_CDR_LE: Self = Self {
        bytes: [0x00, 0x15],
    };
    pub const XML: Self = Self {
        bytes: [0x00, 0x04],
    };
}

pub struct SerializedPayload {
    pub representation_identifier: RepresentationIdentifier,
    pub representation_options: [u8; 2], // Not used. Send as zero, ignore on receive.
    //representation_identifier and representation_options is
    //prescribed by CDR, so cdr crate generate them
    //automaticly
    pub value: Bytes,
}

impl SerializedPayload {
    pub fn from_bytes(buffer: &Bytes) -> io::Result<Self> {
        let mut cursor = io::Cursor::new(&buffer);
        let representation_identifier = RepresentationIdentifier {
            bytes: [cursor.read_u8()?, cursor.read_u8()?],
        };
        let representation_options = [cursor.read_u8()?, cursor.read_u8()?];
        const HEADER_LEN: usize = 4;
        let value = if buffer.len() > HEADER_LEN {
            buffer.slice(HEADER_LEN..)
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "Data is too small"));
        };
        Ok(Self {
            representation_identifier,
            representation_options,
            value,
        })
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.value.len() + 4);
        buf.put_u8(self.representation_identifier.bytes()[0]);
        buf.put_u8(self.representation_identifier.bytes()[1]);
        buf.put_u8(self.representation_options[0]);
        buf.put_u8(self.representation_options[1]);
        buf.put(&self.value[..]);
        buf.freeze()
    }

    pub fn new_from_cdr_data<D: Serialize>(data: D, rep_id: RepresentationIdentifier) -> Self {
        let mut serialized_data = match rep_id {
            RepresentationIdentifier::CDR_LE => {
                cdr::serialize::<_, _, CdrLe>(&data, Infinite).unwrap()
            }
            RepresentationIdentifier::CDR_BE => {
                cdr::serialize::<_, _, CdrBe>(&data, Infinite).unwrap()
            }
            RepresentationIdentifier::PL_CDR_LE => {
                cdr::serialize::<_, _, PlCdrLe>(&data, Infinite).unwrap()
            }
            RepresentationIdentifier::PL_CDR_BE => {
                cdr::serialize::<_, _, PlCdrBe>(&data, Infinite).unwrap()
            }
            _ => unimplemented!(),
        };
        let serialized_rep_id: Vec<_> = serialized_data.drain(0..=1).collect();
        assert_eq!(serialized_rep_id, Vec::from(rep_id.bytes));
        let representation_options = [0; 2];
        let rep_opt: Vec<_> = serialized_data.drain(0..=1).collect();
        assert_eq!(rep_opt, Vec::from(representation_options));
        let value = Bytes::from(serialized_data);
        Self {
            representation_identifier: rep_id,
            representation_options,
            value,
        }
    }
}

pub type GroupDigest = [u8; 4];

impl<C: Context> Writable<C> for SerializedPayload {
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u8(self.representation_identifier.bytes[0])?;
        writer.write_u8(self.representation_identifier.bytes[1])?;
        writer.write_u8(self.representation_options[0])?;
        writer.write_u8(self.representation_options[1])?;
        writer.write_bytes(&self.value)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use cdr::{CdrLe, Infinite};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone)]
    struct Shape {
        color: String,
        x: i32,
        y: i32,
        shapesize: i32,
    }

    #[test]
    fn test_serialize_cdr() {
        let test_shape = Shape {
            color: "BLUE".to_string(),
            x: 42,
            y: 51,
            shapesize: 12,
        };
        let test_serialized = cdr::serialize::<_, _, CdrLe>(&test_shape, Infinite).unwrap();
        const SERIALIZED: [u8; 28] = [
            0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x42, 0x4C, 0x55, 0x45, 0x00, 0x00,
            0x00, 0x00, 0x2A, 0x00, 0x00, 0x00, 0x33, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00,
        ];
        let ser = Vec::from(SERIALIZED);
        assert_eq!(test_serialized, ser);
    }
}
