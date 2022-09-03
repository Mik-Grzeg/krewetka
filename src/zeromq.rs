#[derive(Debug)]
pub struct ZMQSettings {
    pub host: String,
    pub port: u16,
    pub queue: String,
}

impl ZMQSettings {
    pub fn new(host: String, port: u16, queue: String) -> ZMQSettings {
        ZMQSettings {
            host,
            port,
            queue,
        }
    }
}
