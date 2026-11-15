use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn now() -> SystemTime {
    SystemTime::now()
}

pub fn now_secs() -> i64 {
    to_secs(now())
}

/// Unix seconds, the database's instant column type.
pub fn to_secs(t: SystemTime) -> i64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    }
}

pub fn from_secs(s: i64) -> SystemTime {
    if s >= 0 {
        UNIX_EPOCH + Duration::from_secs(s as u64)
    } else {
        UNIX_EPOCH - Duration::from_secs((-s) as u64)
    }
}

pub fn opt_from_secs(s: Option<i64>) -> Option<SystemTime> {
    s.map(from_secs)
}

pub fn opt_to_secs(t: Option<SystemTime>) -> Option<i64> {
    t.map(to_secs)
}
