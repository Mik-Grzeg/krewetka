use super::db::DbLayer;
use super::handlers::get_stats;
use super::handlers::healthz;
use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(healthz);
    cfg.route("/malicious_proportion", web::get().to(get_stats::<DbLayer>));
}
