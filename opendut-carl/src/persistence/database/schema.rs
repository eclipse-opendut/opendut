// @generated automatically by Diesel CLI.

diesel::table! {
    peer (id) {
        id -> Uuid,
        name -> Varchar,
        location -> Nullable<Varchar>,
    }
}
