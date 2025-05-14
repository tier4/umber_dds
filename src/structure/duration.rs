use core::cmp::{Ord, Ordering, PartialOrd};
use core::time::Duration as CoreDuration;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Duration {
    pub seconds: i32,
    pub fraction: u32,
}

impl Duration {
    // TODO
    pub const INFINITE: Self = Self {
        seconds: 0x7fffffff,
        fraction: 0xffffffff,
    };
    pub const ZERO: Self = Self {
        seconds: 0,
        fraction: 0,
    };
    pub fn new(seconds: i32, fraction: u32) -> Self {
        Self { seconds, fraction }
    }

    pub fn half(self) -> Self {
        if self != Self::INFINITE {
            Self {
                seconds: self.seconds / 2,
                fraction: self.fraction / 2,
            }
        } else {
            self
        }
    }

    pub fn to_core_duration(self) -> CoreDuration {
        CoreDuration::new(self.seconds as u64, self.fraction)
    }
}

impl PartialOrd for Duration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Duration {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = (self.seconds as i64) * 1_000_000_000 + (self.fraction as i64);
        let right = (other.seconds as i64) * 1_000_000_000 + (other.fraction as i64);
        left.cmp(&right)
    }
}
