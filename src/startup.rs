use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, middleware::Logger, web::Data, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the connection in a smart pointer
    let db_pool = Data::new(db_pool);
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
            .wrap(Logger::default())
            .service(health_check)
            .service(subscribe)
            // Get a pointer copy and attach it to the application state
            .app_data(db_pool.clone())
    })
    // .bind(address) -- this uses a hard coded address
    .listen(listener)?
    .run();

    Ok(server)
}
