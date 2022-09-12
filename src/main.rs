use rasp_processor::{ApplicationState, init_config};


#[tokio::main]
async fn main() {
    // Setup logger
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    // parse configs
    let (config_cache, config) = init_config().expect("Configuration init failed");
    let app_state = ApplicationState::new(config_cache, config)
        .expect("Unable to initialize application state");

    let _ = ApplicationState::init_components(app_state.config().unwrap()).await;
}
