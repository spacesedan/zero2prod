use actix_web::{get, web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
#[get("/subscriptions/confirm")]
// `actix-web` will only call this route if the `Parameters` are present otherwise will return a
// 400
pub async fn confirm(_parameters: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
