use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, App, HttpServer};
use std::net::TcpListener;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().service(health_check).service(subscribe))
        // .bind(address) -- this uses a hard coded address
        .listen(listener)?
        .run();

    Ok(server)
}
