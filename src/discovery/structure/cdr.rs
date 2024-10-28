/*
 * This code is from https://github.com/hrektts/cdr-rs/blob/master/src/lib.rs
    MIT License

    Copyright (c) 2017 Katsutoshi Horie

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.
*/

//! A serialization/deserialization implementation for Common Data
//! Representation.
//!
//! # Examples
//!
//! ```rust
//! use cdr::{CdrBe, Infinite};
//! use serde_derive::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize, PartialEq)]
//! struct Point {
//!     x: f64,
//!     y: f64,
//! }
//!
//! #[derive(Deserialize, Serialize, PartialEq)]
//! struct Polygon(Vec<Point>);
//!
//! let triangle = Polygon(vec![
//!     Point { x: -1.0, y: -1.0 },
//!     Point { x: 1.0, y: -1.0 },
//!     Point { x: 0.0, y: 0.73 },
//! ]);
//!
//! let encoded = cdr::serialize::<_, _, CdrBe>(&triangle, Infinite).unwrap();
//! let decoded = cdr::deserialize::<Polygon>(&encoded[..]).unwrap();
//!
//! assert!(triangle == decoded);
//! ```

#![deny(clippy::all)]

pub use byteorder::{BigEndian, LittleEndian};

pub mod de;
#[doc(inline)]
pub use self::de::Deserializer;

mod encapsulation;

mod error;
pub use self::error::{Error, Result};

pub mod size;
use std::io::Read;

#[doc(inline)]
pub use self::size::{Infinite, SizeLimit};

/// Deserializes a slice of bytes into an object.
pub fn deserialize<'de, T>(bytes: &[u8]) -> Result<T>
where
    T: serde::Deserialize<'de>,
{
    deserialize_from::<_, _, _>(bytes, Infinite)
}

/// Deserializes an object directly from a `Read`.
pub fn deserialize_from<'de, R, T, S>(reader: R, size_limit: S) -> Result<T>
where
    R: Read,
    T: serde::Deserialize<'de>,
    S: SizeLimit,
{
    use self::encapsulation::ENCAPSULATION_HEADER_SIZE;

    let mut deserializer = Deserializer::<_, S, BigEndian>::new(reader, size_limit);

    let v: [u8; ENCAPSULATION_HEADER_SIZE as usize] =
        serde::Deserialize::deserialize(&mut deserializer)?;
    deserializer.reset_pos();
    match v[1] {
        0 | 2 => serde::Deserialize::deserialize(&mut deserializer),
        1 | 3 => serde::Deserialize::deserialize(
            &mut Into::<Deserializer<_, _, LittleEndian>>::into(deserializer),
        ),
        _ => Err(Error::InvalidEncapsulation),
    }
}
