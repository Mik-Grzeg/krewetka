use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use reporting::app::db_init;
use reporting::app::routes;
use reporting::app::state;

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
    let db = db_init(settings);

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(web::Data::new(db.clone()))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(routes)
    })
    .bind(("127.0.0.1", HTTP_PORT))
    .unwrap_or_else(|_| panic!("unable to bind to port {}", HTTP_PORT))
    .run()
    .await
}
