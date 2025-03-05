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

/// Trait for Data that exchanged via DDS
///
/// UmberDDS specifies exchanged data using a struct instead of IDL.
/// This trait is used for define exchanged data.
///
/// As shown in the following example, implement this trait for your struct by using the provided macro.
/// ```
/// use umber_dds::{DdsData, key::KeyHash};
/// use md5::compute;
/// use cdr::{CdrBe, Infinite}; // If key is specified
///
/// // Struct Shape is same to following idl on other DDS.
/// // struct ShapeType {
/// //     @key string color;
/// //     long x;
/// //     long y;
/// //     long shapesize;
/// // }
///
/// #[derive(DdsData)]
/// #[dds_data(type_name = "ShapeType")]
/// struct Shape {
///     #[key]
///     color: String,
///     x: i32,
///     y: i32,
///     shapesize: i32,
/// }
/// ```
/// You need to import `umber_dds::key::KeyHash` and `md5::compute`.
/// You can specify key to any type that implements the [`Key`] trait.
/// If some key is specified, you need to import `cdr::{CdrBe, Infinite}`
pub trait DdsData {
    fn gen_key(&self) -> KeyHash;
    /// Return type name of Topic.
    ///
    /// The default value is the name of the struct.
    /// If you want to change it, specify `#[dds_data(type_name = "{name}")]`.
    fn type_name() -> String;
    /// Returns whether this type has a key.
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
