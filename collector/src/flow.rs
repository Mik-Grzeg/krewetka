use serde::{Deserializer, Deserialize};

#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct FlowMessageClassBatched {
    #[prost(message, repeated, tag="1")]
    pub classifications: ::prost::alloc::vec::Vec<FlowMessageClass>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlowMessageBatched {
    #[prost(message, repeated, tag="1")]
    pub messages: ::prost::alloc::vec::Vec<FlowMessage>,
}
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
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
    #[serde(deserialize_with = "f32_from_str")]
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

pub fn f32_from_str<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|string| {
        string.parse::<f32>().map_err(|_| Error::custom("failed to deserialize string to f32"))
    })
}
