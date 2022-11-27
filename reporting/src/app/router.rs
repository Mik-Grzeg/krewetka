use super::db::DbLayer;
use super::doc::ApiDoc;
use super::handlers;
use super::ws_handlers;
use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()),
    );
    cfg.route("/healthz", web::get().to(handlers::healthz));
    cfg.route(
        "/grouped_packets_number",
        web::get().to(handlers::get_stats::<DbLayer>),
    );
    cfg.route("/throughput_ws", web::get().to(ws_handlers::throughput_ws::<DbLayer>));
}
