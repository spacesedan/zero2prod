use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{health_check, subscribe},
};
use actix_web::{dev::Server, web::Data, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");

        let timeout = configuration.email_client.timeout();

        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, email_client)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // Wrap the connection in a smart pointer
    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);
    // Capture `connection` from the surrounding environment using `move`
    // HttpServer handles all transport level concerns using a tcp connection that is listening to
    // incoming connections.
    // With it we can:
    //      define where our application should be listening for (port) for incoming requests
    //      Maximum number of concurrent connections that we should allow?
    //      should we enable TLS(transport level security)
    let server = HttpServer::new(move || {
        // `App` is where the application logic is defined, (i.e. what do when a connection hits a
        // certain route, what middle wares to use and how to handelr requests
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(subscribe)
            // Get a pointer copy and attach it to the application state
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    // .bind(address) -- this uses a hard coded address
    .listen(listener)?
    .run();

    Ok(server)
}
