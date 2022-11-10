use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FlowMessage {
    pub out_bytes: u64,
    pub out_pkts: u64,
    pub in_bytes: u64,
    pub in_pkts: u64,
    pub ipv4_src_addr: String,
    pub ipv4_dst_addr: String,
    pub l7_proto: f32,
    pub l4_dst_port: u32,
    pub l4_src_port: u32,
    pub flow_duration_milliseconds: u64,
    pub protocol: u32,
    pub tcp_flags: u32,
}
