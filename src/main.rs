#[macro_use]
extern crate diesel;
use actix_files::Files;
use actix_web::error::ErrorBadRequest;
use actix_web::middleware::Logger;
use actix_web::{
    error, http::header, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use log::{error, info, warn};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use validator::Validate;
use validator_derive::Validate;

use self::models::*;

mod models;
mod schema;
use self::schema::bookss::dsl::*;

mod errors;
use self::errors::UserError;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

async fn book_endpoint(pool: web::Data<DbPool>) -> Result<HttpResponse, UserError> {
    let connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::InternalError
    })?;

    let books_data = web::block(move || bookss.limit(100).load::<Book>(&connection))
        .await
        .map_err(|_| {
            error!("Failed to get books");
            UserError::InternalError
        })?;
    return Ok(HttpResponse::Ok().json(books_data));
}

#[derive(Deserialize, Validate)]
struct BookEndpointPath {
    
    id: i32,
}

async fn bookk_endpoint(
    pool: web::Data<DbPool>,
    book_id: web::Path<BookEndpointPath>,
) -> Result<HttpResponse, UserError> {
    book_id.validate().map_err(|_| {
        warn!("Parameter validation failed");
        UserError::ValidationError
    })?;
    let connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::InternalError
    })?;

    let query_id = book_id.id.clone();
    let book_data = web::block(move || bookss.filter(id.eq(query_id)).first::<Book>(&connection))
        .await
        .map_err(|e| match e {
            error::BlockingError::Error(diesel::result::Error::NotFound) => {
                error!("Book ID: {} not found in DB", &book_id.id);
                UserError::NotFoundError
            }
            _ => {
                error!("Unexpected error");
                UserError::InternalError
            }
        })?;
    Ok(HttpResponse::Ok().json(book_data))
}

async fn book_new(
    pool: web::Data<DbPool>,
    new_bookk: web::Json<NewBook>,
) -> Result<HttpResponse, UserError> {
    
    let connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::InternalError
    })?;

   

    let new_book = NewBook {
        name: new_bookk.name.clone(),
        author: new_bookk.author.clone(),
    };

    let book_data = web::block(move || {
        diesel::insert_into(bookss)
            .values(new_book)
            .execute(&connection)
    })
    .await
    .map_err(|e| match e {
        error::BlockingError::Error(diesel::result::Error::NotFound) => {
            error!("Book ID: not found in DB");
            UserError::NotFoundError
        }
        _ => {
            error!("Unexpected error");
            UserError::InternalError
        }
    })?;
    Ok(HttpResponse::Ok().json(book_data))
}

async fn book_update(
    pool: web::Data<DbPool>,
    new_bookk: web::Json<NewBook>,
    book_id: web::Path<BookEndpointPath>,
) -> Result<HttpResponse, UserError> {
    
    let connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::InternalError
    })?;

    let new_book = NewBook {
        name: new_bookk.name.clone(),
        author: new_bookk.author.clone(),
    };

    let query_id = book_id.id.clone();

    let book_data = web::block(move || {
        diesel::update(bookss.find(query_id))
            .set(&new_book)
            .execute(&connection)
    })
    .await
    .map_err(|e| match e {
        error::BlockingError::Error(diesel::result::Error::NotFound) => {
            error!("Book ID: not found in DB");
            UserError::NotFoundError
        }
        _ => {
            error!("Unexpected error");
            UserError::InternalError
        }
    })?;
    Ok(HttpResponse::Ok().json(book_data))
}

async fn book_delete(
    pool: web::Data<DbPool>,
    book_id: web::Path<BookEndpointPath>,
) -> Result<HttpResponse, UserError> {
   
    let connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::InternalError
    })?;

    

    let query_id = book_id.id.clone();

    let book_data =
        web::block(move || diesel::delete(bookss.filter(id.eq(query_id))).execute(&connection))
            .await
            .map_err(|e| match e {
                error::BlockingError::Error(diesel::result::Error::NotFound) => {
                    error!("Cat ID: not found in DB");
                    UserError::NotFoundError
                }
                _ => {
                    error!("Unexpected error");
                    UserError::InternalError
                }
            })?;
    Ok(HttpResponse::Ok().json(book_data))
}

fn setup_database() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB connection pool.")
}

fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .app_data(
                web::PathConfig::default().error_handler(|_, _| UserError::ValidationError.into()),
            )
            .route("/books", web::get().to(book_endpoint))
            .route("/book/{id}", web::get().to(bookk_endpoint))
            .route("/newbook", web::post().to(book_new))
            .route("/delete/{id}", web::delete().to(book_delete))
            .route("/update/{id}", web::put().to(book_update)),
    );
}

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
