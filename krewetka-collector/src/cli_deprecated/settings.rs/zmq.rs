use super::super::cli::{SettingExt, Args};
use crate::importers::zeromq::ZMQSettings;

impl SettingExt for ZMQSettings { }

impl ZMQSettings {
    pub fn prepare(args: &Args) -> Option<Self> {
        Some(ZMQSettings::new(
            args.zmq_host.clone(),
            args.zmq_port,
            args.zmq_queue_name.clone(),
        ))
    }
}