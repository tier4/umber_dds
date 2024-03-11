use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Duration {
    pub seconds: i32,
    pub fraction: u32,
}

impl Duration {
    // TODO
    pub const INFINITE: Self = Self {
        seconds: 0x7fffffff,
        fraction: 0x7fffffff,
    };
    pub const ZERO: Self = Self {
        seconds: 0,
        fraction: 0,
    };
    pub fn new(seconds: i32, fraction: u32) -> Self {
        Self { seconds, fraction }
    }
}
