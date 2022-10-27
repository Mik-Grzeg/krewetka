use std::time::Duration;

pub const STORAGE_MAX_BUFFER_SIZE: usize = 1024 * 1024;
pub const STORAGE_BUFFER_FLUSH_INTEVAL_IN_SECS: Duration = Duration::from_secs(15);
