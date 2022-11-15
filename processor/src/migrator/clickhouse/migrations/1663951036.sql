CREATE TABLE IF NOT EXISTS messages (
	 host		  	String,
	 out_bytes	    UInt64,
	 out_pkts		UInt64,
	 in_bytes	    UInt64,
	 in_pkts	    UInt64,
	 ipv4_src_addr 	IPv4,
	 ipv4_dst_addr	IPv4,
	 l7_proto		Float32,
	 l4_dst_port	UInt32,
	 l4_src_port	UInt32,
	 flow_duration_milliseconds	UInt64,
	 protocol		UInt32,
	 tcp_flags		UInt32,
	 malicious      Bool
	 timestamp		DateTime
) Engine=MergeTree
ORDER BY (timestamp)
