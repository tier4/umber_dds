use core::cmp::{Ord, Ordering, PartialOrd};
use core::convert::From;
use core::time::Duration as CoreDuration;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Duration {
    /// time in seconds
    pub seconds: i32,
    /// time in sec/2^32
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

    pub fn from_secs(seconds: i32) -> Self {
        Self {
            seconds,
            fraction: 0,
        }
    }

    pub fn from_millis(milliseconds: i64) -> Self {
        let seconds = milliseconds / 1000;
        let frac_sec = (milliseconds - seconds * 1000) as f64 / 1000.;
        let fraction = frac_sec * (1_u64 << 32) as f64;
        Self {
            seconds: seconds as i32,
            fraction: fraction as u32,
        }
    }

    pub fn from_nanos(nanoseconds: i64) -> Self {
        let seconds = nanoseconds / 1_000_000_000;
        let frac_sec = (nanoseconds - seconds * 1_000_000_000) as f64 / 1_000_000_000.;
        let fraction = frac_sec * (1_u64 << 32) as f64;
        Self {
            seconds: seconds as i32,
            fraction: fraction as u32,
        }
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
}

impl From<Duration> for CoreDuration {
    fn from(item: Duration) -> Self {
        CoreDuration::new(
            item.seconds as u64,
            (1_000_000_000 * item.fraction as u64 / (1_u64 << 32)) as u32,
        )
    }
}

impl From<CoreDuration> for Duration {
    fn from(item: CoreDuration) -> Self {
        let nanos = item.subsec_nanos();
        let frac_sec = nanos as f64 / 1_000_000_000.;
        let fraction = frac_sec * (1_u64 << 32) as f64;
        Duration {
            seconds: item.as_secs() as i32,
            fraction: fraction as u32,
        }
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

mod test {
    #[test]
    fn duration_interoperability() {
        use super::Duration;
        use core::time::Duration as CoreDuration;

        // Duration to CoreDuration
        let duration = Duration::from_millis(1500);
        assert_eq!(CoreDuration::from_millis(1500), duration.into());

        // CoreDuration to Duration
        let core_duration = CoreDuration::from_millis(1500);
        assert_eq!(Duration::from_millis(1500), Duration::from(core_duration));
    }
}
