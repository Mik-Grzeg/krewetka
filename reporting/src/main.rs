use actix_web::{App, HttpServer};
use lib::app::routes;
use lib::app::state;

const HTTP_PORT: u16 = 8080;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // Setup logger
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    let state = state::State::new()
        .await
        .expect("unable to initialize app state");
    HttpServer::new(move || App::new().app_data(state.clone()).configure(routes))
        .bind(("127.0.0.1", HTTP_PORT))
        .unwrap_or_else(|_| panic!("unable to bind to port {}", HTTP_PORT))
        .run()
        .await
}
