use actix_web::{http, HttpResponse, Responder};
use utoipa;

/// Health check endpoint
///
/// Checks whether the server is capable of responding to a request
#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Api is healthy", body = String),
    ),
)]
pub async fn healthz() -> impl Responder {
    HttpResponse::build(http::StatusCode::OK).body("OK".to_owned())
}
