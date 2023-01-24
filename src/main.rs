use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // trace -> debug -> info -> warn -> error // log level severtity
    // If no RUST_LOG environment variable has been set the value will default to `info`
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // Panic if we can't read configuration
    // get the configuration needed from file.
    let configuration = get_configuration().expect("Failed to read configuration.");
    // get a connection to Postgres.
    let connection_pool = PgPool::connect(&configuration.database.connetion_string())
        .await
        .expect("Failed to connect to Postgres");
    // format our address in order to give it to our TCP listener.
    // could do this inside of the `bind()` but chose to do define it on its own for readability
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    println!(
        "listening to request on port: {}",
        configuration.application_port
    );
    // start our app using the `TcpListener` and `PgPool`.
    run(listener, connection_pool)?.await
}
