use actix_web::{post, web, HttpResponse};
use serde::Deserialize;

// to use Deserialize like this you have to enable the derive feature on serde.
#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions")]
pub async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
