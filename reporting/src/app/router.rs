use super::handlers::healthz;
use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(healthz);
}
