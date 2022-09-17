fn main() {

    let proto_path = match std::env::var("PROTO_PATH") {
        Ok(val) => val,
        Err(_) => "../proto/flow.proto".to_string(),
    };

    tonic_build::compile_protos(&proto_path)
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
