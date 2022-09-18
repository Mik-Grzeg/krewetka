fn main() {
    let proto_path = match std::env::var("PROTO_PATH") {
        Ok(val) => val,
        Err(_) => "proto/flow.proto".to_string(),
    };
    let mut proto_path_ancestors = std::path::Path::new(&proto_path).ancestors();

    tonic_build::configure()
        .type_attribute(
            ".flow.FlowMessage",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flow.FlowMessage",
            "#[serde(rename_all(deserialize = \"SCREAMING_SNAKE_CASE\"))]",
        )
        .compile(&[&proto_path_ancestors.next().unwrap()], &[&proto_path_ancestors.next().unwrap()])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
