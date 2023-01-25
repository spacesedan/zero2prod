use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

// to use Deserialize like this you have to enable the derive feature on serde.
#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name= "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_name = %form.email,
        subscriber_name = %form.name
        )
    )]
#[post("/subscriptions")]
pub async fn subscribe(
    form: web::Form<FormData>,
    // Retrieving a connection from the application state!
    pool: web::Data<PgPool>,
) -> HttpResponse {
    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
                 INSERT INTO subscriptions (id, email, name, subscribed_at)
                 VALUES ($1, $2, $3, $4)

                 "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to excecute query: {:?}", e);
        e
    })?;
    Ok(())
}