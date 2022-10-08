use lib::application_state::ApplicationState;
use log::info;


use lib::pb::{flow_message_classifier_client::FlowMessageClassifierClient, FlowMessage, FlowMessageClass};
pub mod pb {
    tonic::include_proto!("flow");
}

#[tokio::main]
async fn main() {
    // Setup logger
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    info!("Starting application");

    // TODO change to proper client initialization

    // println!("classification");
    // streaming_classifier(&mut client, 10).await;
    

    let state = match ApplicationState::new().await {
        Ok(s) => s,
        Err(e) => panic!("ApplicationState init error: {:?}", e),
    };

    state.init().await;
}