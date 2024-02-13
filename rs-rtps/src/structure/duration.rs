use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
pub struct Duration {
    pub seconds: i32,
    pub fraction: u32,
}

impl Duration {
    pub fn new(seconds: i32, fraction: u32) -> Self {
        Self { seconds, fraction }
    }
}
