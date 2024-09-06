use byteorder::{BigEndian, ByteOrder, LittleEndian};

pub const ENCAPSULATION_HEADER_SIZE: u64 = 4;

/// Data encapsulation scheme identifiers.
pub trait Encapsulation {
    type E: ByteOrder;
    const ID: [u8; 2];
    const OPTION: [u8; 2] = [0; 2];
}

/// OMG CDR big-endian encapsulation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CdrBe {}

impl Encapsulation for CdrBe {
    type E = BigEndian;
    const ID: [u8; 2] = [0, 0];
}

/// OMG CDR little-endian encapsulation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CdrLe {}

impl Encapsulation for CdrLe {
    type E = LittleEndian;
    const ID: [u8; 2] = [0, 1];
}

/// ParameterList encapsulated using OMG CDR big-endian encapsulation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PlCdrBe {}

impl Encapsulation for PlCdrBe {
    type E = BigEndian;
    const ID: [u8; 2] = [0, 2];
}

/// ParameterList encapsulated using OMG CDR little-endian encapsulation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PlCdrLe {}

impl Encapsulation for PlCdrLe {
    type E = LittleEndian;
    const ID: [u8; 2] = [0, 3];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant() {
        assert_eq!(
            ENCAPSULATION_HEADER_SIZE,
            (CdrBe::ID.len() + CdrBe::OPTION.len()) as u64
        );
        assert_eq!(
            ENCAPSULATION_HEADER_SIZE,
            (CdrLe::ID.len() + CdrLe::OPTION.len()) as u64
        );
        assert_eq!(
            ENCAPSULATION_HEADER_SIZE,
            (PlCdrBe::ID.len() + PlCdrBe::OPTION.len()) as u64
        );
        assert_eq!(
            ENCAPSULATION_HEADER_SIZE,
            (PlCdrLe::ID.len() + PlCdrLe::OPTION.len()) as u64
        );
    }
}
