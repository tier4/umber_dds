use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
pub struct Duration {
    pub seconds: i32,
    pub fraction: u32,
}
