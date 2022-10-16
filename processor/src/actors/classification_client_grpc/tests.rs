#[cfg(test)]
mod tests {
    use supper::*;
    use mockall::mock;
    use pretty_assertions::assert_eq;
    use tokio_test::block_on;
    use tokio::sync::{broadcast, mpsc};
    
    fn generate_flow_messages_with_metadata() {
        FlowMessage {
            out_bytes: 1 + i as u64,
            out_pkts: 2,
            in_bytes: 3,
            in_pkts: 4,
            ipv4_src_addr: "192.168.1.1".into(),
            ipv4_dst_addr: "192.168.1.2".into(),
            l7_proto: 7.2,
            l4_dst_port: 5507,
            l4_src_port: 4091,
            flow_duration_milliseconds: 123,
            protocol: 7,
            tcp_flags: 11,
        }
    }


    #[test]
    fn test_streaming_classifier_sends_output() {
        todo!()
    }
}
