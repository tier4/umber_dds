use keyed_derive::Keyed;
use serde::ser::Serialize;

#[derive(Debug)]
pub struct KeyHash {
    hash: [u8; 16],
}

impl KeyHash {
    fn new(bytes: &[u8]) -> Self {
        let mut hash_in = [0u8; 16];
        hash_in.copy_from_slice(&bytes);
        Self { hash: hash_in }
    }
}

pub trait Keyed {
    fn gen_key(&self) -> KeyHash;
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

mod test {
    use super::{Key, KeyHash, Keyed};
    use cdr::{calc_serialized_size, CdrBe, Infinite};
    use md5::compute;

    #[derive(Keyed, Debug)]
    struct Shape {
        #[key]
        color: String,
        x: i32,
        y: i32,
        #[key]
        shapesize: i32,
    }

    #[test]
    fn test() {
        let shape = Shape {
            color: String::from("red"),
            x: 10,
            y: 20,
            shapesize: 30,
        };

        let keyhash = shape.gen_key();
        // TODO: check keyhash is correct
        panic!("{:?}", keyhash); // for check keyhash value
    }
}