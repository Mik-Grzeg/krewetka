use lib::application_state::ApplicationState;
use log::info;

#[tokio::main]
async fn main() {
    // Setup logger
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    info!("Starting application");
    let state = match ApplicationState::new() {
        Ok(s) => s,
        Err(e) => panic!("ApplicationState init error"),
    };

    state.init().await
}
