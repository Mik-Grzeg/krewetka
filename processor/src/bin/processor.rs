use actix_web::{App, HttpServer};
use log::info;
use processor::application_state::ApplicationState;
use processor::consts::HTTP_PORT;
use processor::handler::healthz;

pub mod pb {
    include!("../flow.rs");
}

#[actix_web::main]
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

    HttpServer::new(|| App::new().service(healthz))
        .bind(format!("0.0.0.0:{}", HTTP_PORT))
        .unwrap_or_else(|_| panic!("unable to bind to port {}", HTTP_PORT))
        .run()
        .await;
}
