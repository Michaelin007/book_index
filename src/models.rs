use super::schema::bookss;
use actix_web::web;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize)]
pub struct Book {
    pub id: i32,
    pub name: String,
    pub author: String,
}

#[derive(Debug, Insertable, AsChangeset, Serialize, Deserialize, Clone)]
#[table_name = "bookss"]
pub struct NewBook {
    
    pub name: String,
    pub author: String,
}
impl From<web::Json<NewBook>> for NewBook {
    fn from(book: web::Json<NewBook>) -> Self {
        NewBook {
            name: book.name.clone(),
            author: book.author.clone(),
        }
    }
}
