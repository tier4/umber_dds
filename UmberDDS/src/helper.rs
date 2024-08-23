static mut NOW: fn() -> Option<i64> = default_now;

#[cfg(feature = "std")]
fn default_now() -> Option<i64> {
    chrono::Local::now().timestamp_nanos_opt()
}

#[cfg(not(feature = "std"))]
fn default_now() -> Option<i64> {
    None
}

/// Set the function to be used to get the current time.
///
/// # Safety
///
/// `now_fn` must return the number of non-leap-nanoseconds since January 1, 1970 UTC.
pub unsafe fn set_now_fn(now_fn: fn() -> Option<i64>) {
    unsafe {
        NOW = now_fn;
    }
}

/// Get the current time in nanoseconds since January 1, 1970 UTC.
pub fn now() -> Option<i64> {
    unsafe { NOW() }
}
