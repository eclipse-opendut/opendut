// @generated automatically by Diesel CLI.

diesel::table! {
    peer_descriptor (peer_id) {
        peer_id -> Uuid,
        name -> Varchar,
        location -> Nullable<Varchar>,
    }
}
