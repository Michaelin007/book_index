// @generated automatically by Diesel CLI.

diesel::table! {
    books (id) {
        id -> Int4,
        name -> Varchar,
        author -> Varchar,
    }
}

diesel::table! {
    bookss (id) {
        id -> Int4,
        name -> Varchar,
        author -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(books, bookss,);
