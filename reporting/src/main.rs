use actix_web::{App, HttpServer};
use lib::app::db;
use lib::app::routes;
use lib::app::state;
use std::sync::Arc;

const HTTP_PORT: u16 = 8080;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // Setup logger
    let env = env_logger::Env::default();
    env_logger::init_from_env(env);

    let state = state::State::new()
        .await
        .expect("unable to initialize app state");
    let settings = state.config()?;
    let db = Arc::new(db::init(settings));

    HttpServer::new(move || App::new().app_data(db.clone()).configure(routes))
        .bind(("127.0.0.1", HTTP_PORT))
        .unwrap_or_else(|_| panic!("unable to bind to port {}", HTTP_PORT))
        .run()
        .await
}
