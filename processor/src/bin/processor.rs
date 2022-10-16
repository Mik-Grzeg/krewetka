use lib::application_state::ApplicationState;
use log::info;

pub mod pb {
    include!("../flow.rs");
}

#[actix::main]
async fn main() {
    // Setup logger
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    info!("Starting application");

    let state = match ApplicationState::new().await {
        Ok(s) => s,
        Err(e) => panic!("ApplicationState init error: {:?}", e),
    };

    state.init_actors().await;
}
