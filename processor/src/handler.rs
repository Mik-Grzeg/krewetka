use actix_web::{get, http, HttpResponse, Responder};

#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::build(http::StatusCode::OK).body("OK".to_owned())
}
