// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "network_interface_kind"))]
    pub struct NetworkInterfaceKind;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::NetworkInterfaceKind;

    network_interface_descriptor (network_interface_id) {
        network_interface_id -> Uuid,
        name -> Text,
        kind -> NetworkInterfaceKind,
        peer_id -> Uuid,
    }
}

diesel::table! {
    network_interface_kind_can (network_interface_id) {
        network_interface_id -> Uuid,
        bitrate -> Int4,
        sample_point_times_1000 -> Int4,
        fd -> Bool,
        data_bitrate -> Int4,
        data_sample_point_times_1000 -> Int4,
    }
}

diesel::table! {
    peer_descriptor (peer_id) {
        peer_id -> Uuid,
        name -> Text,
        location -> Nullable<Text>,
        network_bridge_name -> Nullable<Text>,
    }
}

diesel::joinable!(network_interface_descriptor -> peer_descriptor (peer_id));
diesel::joinable!(network_interface_kind_can -> network_interface_descriptor (network_interface_id));

diesel::allow_tables_to_appear_in_same_query!(
    network_interface_descriptor,
    network_interface_kind_can,
    peer_descriptor,
);
