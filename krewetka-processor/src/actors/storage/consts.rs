use std::time::Duration;

pub const BASE_RETRY_STORAGE_INSERT_INTERVAL_IN_SECS: u64 = 3;
pub const RETRY_STORAGE_INSERT_INTERVAL_IN_SECS: &[Duration; 3] = &[
    Duration::from_secs(BASE_RETRY_STORAGE_INSERT_INTERVAL_IN_SECS),
    Duration::from_secs(BASE_RETRY_STORAGE_INSERT_INTERVAL_IN_SECS.pow(2)),
    Duration::from_secs(BASE_RETRY_STORAGE_INSERT_INTERVAL_IN_SECS.pow(3)),
];
