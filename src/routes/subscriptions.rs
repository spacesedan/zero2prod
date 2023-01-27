use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

// to use Deserialize like this you have to enable the derive feature on serde.
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// Creates a `NewSubscriber` from `FormData` using the TryFrom trait. similar functionality could be
// accomplished using `TryInto`
impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

// Telemetry data lets us get better insight as too what is going on in our application.
#[tracing::instrument(
    name= "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
        )
    )]
#[post("/subscriptions")]
pub async fn subscribe(
    form: web::Form<FormData>,
    // Retrieving a connection from the application state! this is the way `actix_web` handles
    // dependency injection
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // create a `new_subscriber` from teh incoming form
    let new_subscriber = match form.0.try_into() {
        // if the information passed in the form passes our validation then we create a
        // `new_subscriber` that can be stored in our db.
        Ok(form) => form,
        // if the information on the form is invalid return a `400 BAD REQUEST`
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    // Try to insert the `new_subscriber` into our db.
    match insert_subscriber(&pool, &new_subscriber).await {
        // if the `new_subscriber` does not already exist, we store them oin our db.
        Ok(_) => HttpResponse::Ok().finish(),
        // if anything does wrong when trying to store a subscriber return a `500 INTERNAL SERVER
        // ERROR`
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
                 INSERT INTO subscriptions (id, email, name, subscribed_at)
                 VALUES ($1, $2, $3, $4)

                 "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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
