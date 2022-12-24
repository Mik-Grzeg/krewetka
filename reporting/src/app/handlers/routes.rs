use super::doc::ApiDoc;
use super::{flow_stats, flows_details_ws, health, throughput_ws};
use crate::app::db::DbLayer;
use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()),
    );
    cfg.route("/healthz", web::get().to(health::healthz));
    cfg.route(
        "/grouped_packets_number",
        web::get().to(flow_stats::get_stats::<DbLayer>),
    );
    cfg.route(
        "/throughput",
        web::get().to(throughput_ws::stream_throughput::<DbLayer>),
    );
    cfg.route(
        "/flows_details",
        web::get().to(flows_details_ws::stream_flows_detail::<DbLayer>),
    );
}
