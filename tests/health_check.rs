use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::run,
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
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    // get the port address of the newly created listener
    let port = listener.local_addr().unwrap().port();
    // load configuration file
    let mut configuration = get_configuration().expect("Failed to load configuration file");
    // change the datbase name used in the db.
    configuration.database.database_name = Uuid::new_v4().to_string();
    // configure connection pool using the randmonly craeted db name.
    let connection_pool = configure_database(&configuration.database).await;
    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");

    let _ = tokio::spawn(server);
    // Don't forget to put the `http` or won't work lol.
    // return the port in a formatted string that can be used in unit tests.
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Connect to Postgres without using a default db name
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");

    // create a new db
    connection
        .execute(
            format!(
                r#"
                CREATE DATABASE "{}";
                "#,
                config.database_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to create database");

    // Create a connection pool using the newly created db.
    let connection_pool = PgPool::connect_with(config.without_db())
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

// actix_rt::test is the testing equivalent of `actix_web::main`
#[actix_rt::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    // use reqwest crate to perform HTTP requests against our application
    let client = reqwest::Client::new();

    let response = client
        //  use the address returned from the newly created test app.
        .get(format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    // assert_eq!(Some(0), response.content_length())
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_data() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=spacedaddy&email=space_daddy%40test.com";

    // Act
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "space_daddy@test.com");
    assert_eq!(saved.name, "spacedaddy")
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=space%20daddy", "missing the email"),
        ("email=space_daddy%40test.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    // Act
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with a 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}
