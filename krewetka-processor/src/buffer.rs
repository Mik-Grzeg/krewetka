use flow::FlowMessage;
use tokio::time::Duration;

pub struct StateBuffer {
    buffer: Vec<FlowMessage>,
    flush_interval_in_secs: Duration,
}

// impl StateBuffer {
//     fn new()
// }
