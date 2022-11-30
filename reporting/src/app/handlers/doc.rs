use super::{flow_stats, health};
use crate::app::models::MaliciousVsNonMalicious;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        flow_stats::get_stats,
        health::healthz,
    ),
    components(
        schemas(MaliciousVsNonMalicious),
    ),
    tags(
        (name = "reporting", description = "Reporting api endpoints")
    ),
)]
pub struct ApiDoc;
