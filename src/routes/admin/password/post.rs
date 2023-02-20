use actix_web::{post, web, HttpResponse};
use secrecy::{ExposeSecret, Secret};

use crate::utils::see_other;

#[derive(serde::Deserialize)]
pub struct FormData {
    // current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

#[post("/admin/password")]
pub async fn change_password(form: web::Form<FormData>) -> Result<HttpResponse, actix_web::Error> {
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        return Ok(see_other("/admin/password"));
    }
    todo!()
}
