use serde::ser::Serialize;

#[derive(Debug)]
pub struct KeyHash {
    _hash: [u8; 16],
}

impl KeyHash {
    pub fn new(bytes: &[u8]) -> Self {
        let mut hash_in = [0u8; 16];
        hash_in.copy_from_slice(bytes);
        Self { _hash: hash_in }
    }
}

pub trait DdsData {
    fn gen_key(&self) -> KeyHash;
    fn type_name() -> String;
    fn is_with_key() -> bool;
}

pub trait Key: std::fmt::Debug + Serialize {}

impl Key for bool {} // IDL: boolean
impl Key for char {} // IDL: char
impl Key for u8 {} // IDL: octet
impl Key for i16 {} // IDL: short
impl Key for u16 {} // IDL: unsigned short
impl Key for i32 {} // IDL: long
impl Key for u32 {} // IDL: unsigned long
impl Key for i64 {} // IDL: long long
impl Key for u64 {} // IDL: unsigned long long
impl Key for f32 {} // IDL: float
impl Key for f64 {} // IDL: double

impl Key for String {} // IDL: String
impl<K: Key> Key for Vec<K> {} // IDL: sequence<K: Key>

#[cfg(test)]
mod test {
    use super::KeyHash;
    use crate::DdsData;
    use cdr::{CdrBe, Infinite};
    use md5::compute;

    #[derive(DdsData, Debug)]
    struct Shape {
        #[key]
        color: String,
        _x: i32,
        _y: i32,
        _shapesize: i32,
    }

    #[test]
    fn test() {
        let shape = Shape {
            color: String::from("RED"),
            _x: 10,
            _y: 20,
            _shapesize: 30,
        };

        let keyhash = shape.gen_key();
        assert_eq!(
            keyhash._hash,
            [
                0x00, 0x00, 0x00, 0x04, 0x52, 0x45, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ]
        )
    }
}
