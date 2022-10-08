#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlowMessageClass {
    #[prost(bool, tag="1")]
    pub malicious: bool,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlowMessage {
    #[prost(uint64, tag="1")]
    pub out_bytes: u64,
    #[prost(uint64, tag="2")]
    pub out_pkts: u64,
    /// In sizes of packets
    #[prost(uint64, tag="3")]
    pub in_bytes: u64,
    #[prost(uint64, tag="4")]
    pub in_pkts: u64,
    /// Source/destination addresses
    #[prost(string, tag="5")]
    pub ipv4_src_addr: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub ipv4_dst_addr: ::prost::alloc::string::String,
    /// Layer 7 protocol
    #[prost(float, tag="7")]
    pub l7_proto: f32,
    /// Layer 4 port
    #[prost(uint32, tag="8")]
    pub l4_dst_port: u32,
    #[prost(uint32, tag="9")]
    pub l4_src_port: u32,
    /// Duration
    #[prost(uint64, tag="10")]
    pub flow_duration_milliseconds: u64,
    /// Protocol
    #[prost(uint32, tag="11")]
    pub protocol: u32,
    /// TCP flags
    #[prost(uint32, tag="12")]
    pub tcp_flags: u32,
}
