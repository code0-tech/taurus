//! Shared time helpers for runtime metadata.

use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix_micros() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|it| it.as_micros() as i64)
        .unwrap_or(0)
}
