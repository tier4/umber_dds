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

use crate::structure::parameter_id::ParameterId;
use byteorder::ReadBytesExt;
use bytes::Bytes;
use cdr::{CdrLe, Infinite};
use serde::Serialize;
use speedy::{Context, Readable, Reader, Writable, Writer};
use std::io;
use std::ops::{Add, AddAssign};

// spec 9.4.2 Mapping of the PIM SubmessageElements

pub type Count = i32;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
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
pub type FragmentNumberSet = NumberSet<FragmentNumber>;

#[derive(Readable, Writable)]
pub struct NumberSet<T> {
    bitmap_base: T,
    num_bits: u32,
    bitmap: Vec<u32>,
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
    seconds: u32,
    fraction: u32,
}

impl Timestamp {
    pub const TIME_INVALID: Self = Self {
        seconds: 0x00,
        fraction: 0x00,
    };
}

// spec versin 2.3 9.3.2 Mapping of the Types that Appear Within Submessages or Built-in Topic Data
#[derive(Readable, Writable)]
pub struct Locator {
    kind: i64,
    port: u64,
    address: [u8; 16],
}

impl Locator {
    pub const KIND_INVALID: i64 = -1;
    pub const KIND_RESERVED: i64 = 0;
    pub const KIND_UDPV4: i64 = 1;
    pub const KIND_UDPV6: i64 = 2;
    pub const PORT_INVALID: u64 = 0;
    pub const ADDRESS_INVALID: [u8; 16] = [0; 16];
    pub const INVALID: Self = Self {
        kind: Self::KIND_INVALID,
        port: Self::PORT_INVALID,
        address: Self::ADDRESS_INVALID,
    };
}
pub type LocatorList = Vec<Locator>;

#[derive(Readable, Writable)]
pub struct RepresentationIdentifier {
    bytes: [u8; 2],
}

impl RepresentationIdentifier {
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

    pub fn new_from_cdr_data<D: Serialize>(data: D) -> Self {
        let mut serialized_data = cdr::serialize::<_, _, CdrLe>(&data, Infinite).unwrap();
        let representation_identifier = RepresentationIdentifier::CDR_LE;
        let rep_id: Vec<_> = serialized_data.drain(0..=1).collect();
        assert_eq!(rep_id, Vec::from(RepresentationIdentifier::CDR_LE.bytes));
        let representation_options = [0; 2];
        let rep_opt: Vec<_> = serialized_data.drain(0..=1).collect();
        assert_eq!(rep_opt, Vec::from(representation_options));
        let value = Bytes::from(serialized_data);
        Self {
            representation_identifier,
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
