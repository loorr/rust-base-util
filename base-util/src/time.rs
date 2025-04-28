use std::time::SystemTime;

pub fn current_time_nanos() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("don't use clocks from before 1970")
        .as_nanos()
        .try_into()
        .unwrap_or(0)
}

pub fn current_time_micros() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("don't use clocks from before 1970")
        .as_micros()
        .try_into()
        .unwrap_or(0)
}

pub fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("don't use clocks from before 1970")
        .as_millis()
        .try_into()
        .unwrap_or(0)
}

pub fn current_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("don't use clocks from before 1970")
        .as_secs()
}
