use crate::app::models::MaliciousVsNonMalicious;
use super::{health, flow_stats};
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
