use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".into();
    let subscriber_name = "test".into();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

// Test application defintion that will be used to run our mailing list against.
// it consists of an addres which is the port the test app is runnign in,
// and a `PgPool` in order to be able to manage multiple queries happening concurrently while the
// tests are being ran
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

// Launches our application in the background
// `spawn_app` is a test helper functions that create an instance of our app to run tests against.
// - we create a listener and bind it to port `0` which will give us a random open port that we can
// use
// - We get the port from the listener which will be used later in order to run our tests
// - we then bring in our configuration in order to connection in order to get the necessary
// information to connect our db instance
// - we change the name of our db in our database config to a random string this will make it so we
// can rerun our tests without having to stop and start our Postgres image after every test
// - We get a `PgPool` using the updated configuration and return a connection to our database that
// we can use to run tests
// - We start out `App` using the listener and the connection_pool and start listening for
// requests.
// - we move our application to a different thread have it run its test and once complete it will
// close out app.
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    // Randomize configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        // create a new random test so test do not `collide`
        c.database.database_name = Uuid::new_v4().to_string();
        // set application port to `0` so that the os can choose a random port that is not being
        // used.
        c.application.port = 0;
        // set the `base_url` of the `email_client` to the uri of the mock server.
        c.email_client.base_url = email_server.uri();
        // Return the new randomized configuration
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");

    let address = format!("http://127.0.0.1:{}", application.port());

    let _ = tokio::spawn(application.run_until_stopped());
    // Don't forget to put the `http` or won't work.
    // return the port in a formatted string that can be used in unit tests.
    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Connect to Postgres without using a default db name
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");

    // create a new db
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database");

    // Create a connection pool using the newly created db.
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");

    // use our migrations to init our tables on the
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    // return the connection pool
    connection_pool
}
