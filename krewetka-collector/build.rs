fn main() {
    tonic_build::configure()
        .type_attribute(
            ".flow.FlowMessage",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flow.FlowMessage",
            "#[serde(rename_all(deserialize = \"SCREAMING_SNAKE_CASE\"))]",
        )
        .compile(&["src/proto/flow.proto"], &["/src/proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
