use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use log::{error, warn};
use serde::Deserialize;
use validator::Validate;
use validator_derive::Validate;

use crate::db::DbPool;
use crate::errors::UserError;
use crate::models::{Book, NewBook};
use crate::schema::books::dsl::*;

#[derive(Deserialize, Validate)]
struct BookEndpointPath {
    id: i32,
}

async fn get_books(pool: web::Data<DbPool>) -> Result<HttpResponse, UserError> {
    let mut connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::InternalError
    })?;

    let books_data = web::block(move || books.limit(100).load::<Book>(&mut connection))
        .await
        .map_err(|_| UserError::InternalError)?
        .map_err(|e| match e {
            diesel::result::Error::NotFound => UserError::NotFoundError,
            _ => UserError::InternalError,
        })?;
    Ok(HttpResponse::Ok().json(books_data))
}

async fn get_book(
    pool: web::Data<DbPool>,
    book_id: web::Path<BookEndpointPath>,
) -> Result<HttpResponse, UserError> {
    book_id.validate().map_err(|_| {
        warn!("Parameter validation failed");
        UserError::ValidationError
    })?;
    let mut connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::InternalError
    })?;

    let query_id = book_id.id;
    let book_data =
        web::block(move || books.filter(id.eq(query_id)).first::<Book>(&mut connection))
            .await
            .map_err(|_| UserError::InternalError)?
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    error!("Book ID: {} not found in DB", &book_id.id);
                    UserError::NotFoundError
                }
                _ => UserError::InternalError,
            })?;

    Ok(HttpResponse::Ok().json(book_data))
}

async fn create_book(
    pool: web::Data<DbPool>,
    new_book: web::Json<NewBook>,
) -> Result<HttpResponse, UserError> {
    let mut connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::InternalError
    })?;

    let book_data = web::block(move || {
        diesel::insert_into(books)
            .values(&*new_book)
            .get_result::<Book>(&mut connection)
    })
    .await
    .map_err(|_| UserError::InternalError)?
    .map_err(|e| match e {
        diesel::result::Error::NotFound => {
            error!("Book ID: not found in DB");
            UserError::NotFoundError
        }
        _ => UserError::InternalError,
    })?;

    Ok(HttpResponse::Ok().json(book_data))
}

async fn update_book(
    pool: web::Data<DbPool>,
    updated_book: web::Json<NewBook>,
    book_id: web::Path<BookEndpointPath>,
) -> Result<HttpResponse, UserError> {
    let mut connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::InternalError
    })?;

    let query_id = book_id.id;

    let book_data = web::block(move || {
        diesel::update(books.find(query_id))
            .set(&*updated_book)
            .get_result::<Book>(&mut connection)
    })
    .await
    .map_err(|_| UserError::InternalError)?
    .map_err(|e| match e {
        diesel::result::Error::NotFound => {
            error!("Book ID: not found in DB");
            UserError::NotFoundError
        }
        _ => UserError::InternalError,
    })?;

    Ok(HttpResponse::Ok().json(book_data))
}

async fn delete_book(
    pool: web::Data<DbPool>,
    book_id: web::Path<BookEndpointPath>,
) -> Result<HttpResponse, UserError> {
    let mut connection = pool.get().map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::InternalError
    })?;

    let query_id = book_id.id;

    let book_data =
        web::block(move || diesel::delete(books.filter(id.eq(query_id))).execute(&mut connection))
            .await
            .map_err(|_| UserError::InternalError)?
            .map_err(|e| match e {
                diesel::result::Error::NotFound => {
                    error!("Book ID: not found in DB");
                    UserError::NotFoundError
                }
                _ => UserError::InternalError,
            })?;

    Ok(HttpResponse::Ok().json(book_data))
}

pub(crate) fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .app_data(
                web::PathConfig::default().error_handler(|_, _| UserError::ValidationError.into()),
            )
            .route("/books", web::get().to(get_books))
            .route("/book/{id}", web::get().to(get_book))
            .route("/book", web::post().to(create_book))
            .route("/book/{id}", web::delete().to(delete_book))
            .route("/book/{id}", web::put().to(update_book)),
    );
}
