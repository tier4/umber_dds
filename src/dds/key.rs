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

impl Key for bool {}
impl Key for char {}
impl Key for i8 {}
impl Key for u8 {}
impl Key for i16 {}
impl Key for u16 {}
impl Key for i32 {}
impl Key for u32 {}
impl Key for i64 {}
impl Key for u64 {}

impl Key for String {}

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
