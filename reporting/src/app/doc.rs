use super::handlers;
use super::models::MaliciousVsNonMalicious;
use utoipa::OpenApi;

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
