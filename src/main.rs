use zero2prod::configuration::get_configuration;
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // trace -> debug -> info -> warn -> error // log level severtity
    // If no RUST_LOG environment variable has been set the value will default to `info`
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we can't read configuration
    // get the configuration needed from file.
    let configuration = get_configuration().expect("Failed to read configuration");

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;

    Ok(())
}
