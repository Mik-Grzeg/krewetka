fn main() {
    let proto_path = match std::env::var("PROTO_PATH") {
        Ok(val) => val,
        Err(_) => "./proto/flow.proto".to_string(),
    };
    let mut proto_path_ancestors = std::path::Path::new(&proto_path).ancestors();

    tonic_build::configure()
        .build_server(false)
        .out_dir("./src")
        .compile(
            &[&proto_path_ancestors.next().unwrap()],
            &[&proto_path_ancestors.next().unwrap()],
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
