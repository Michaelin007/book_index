#[macro_use]
extern crate diesel;

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use log::info;

mod db;
mod errors;
mod handlers;
mod models;
mod schema;

use db::setup_database;
use handlers::api_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let pool = setup_database();

    info!("Listening on port 8080");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(pool.clone())
            .configure(api_config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
