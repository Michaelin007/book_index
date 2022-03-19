table! {
    bookss (id) {
        id -> Int4,
        name -> Varchar,
        author -> Varchar,
    }
}

table! {
    cats (id) {
        id -> Int4,
        name -> Varchar,
        image_path -> Varchar,
    }
}

table! {
    catss (id) {
        id -> Int4,
        name -> Varchar,
        image_path -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(bookss, cats, catss,);
