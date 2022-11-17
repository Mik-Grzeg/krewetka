use utoipa::OpenApi;

use super::handlers;
use super::models::MaliciousVsNonMalicious;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::get_stats,
        handlers::healthz,
    ),
    components(
        schemas(MaliciousVsNonMalicious),
    ),
    tags(
        (name = "reporting", description = "Reporting api endpoints")
    ),
)]
pub struct ApiDoc;
