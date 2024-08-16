// @generated automatically by Diesel CLI.

diesel::table! {
    cluster_configuration (cluster_id) {
        cluster_id -> Uuid,
        name -> Text,
        leader_id -> Uuid,
    }
}

diesel::table! {
    cluster_deployment (cluster_id) {
        cluster_id -> Uuid,
    }
}

diesel::table! {
    cluster_device (cluster_id, device_id) {
        cluster_id -> Uuid,
        device_id -> Uuid,
    }
}

diesel::table! {
    device_descriptor (device_id) {
        device_id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        network_interface_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    device_tag (device_id, name) {
        device_id -> Uuid,
        name -> Text,
    }
}

diesel::table! {
    executor_descriptor (executor_id) {
        executor_id -> Uuid,
        kind -> Text,
        results_url -> Nullable<Text>,
        peer_id -> Uuid,
    }
}

diesel::table! {
    executor_kind_container (executor_id) {
        executor_id -> Uuid,
        engine -> Text,
        name -> Nullable<Text>,
        image -> Text,
        volumes -> Array<Nullable<Text>>,
        devices -> Array<Nullable<Text>>,
        envs -> Array<Nullable<Jsonb>>,
        ports -> Array<Nullable<Text>>,
        command -> Nullable<Text>,
        args -> Array<Nullable<Text>>,
    }
}

diesel::table! {
    network_interface_descriptor (network_interface_id) {
        network_interface_id -> Uuid,
        name -> Text,
        kind -> Text,
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

diesel::joinable!(cluster_configuration -> peer_descriptor (leader_id));
diesel::joinable!(cluster_deployment -> cluster_configuration (cluster_id));
diesel::joinable!(cluster_device -> cluster_configuration (cluster_id));
diesel::joinable!(cluster_device -> device_descriptor (device_id));
diesel::joinable!(device_descriptor -> network_interface_descriptor (network_interface_id));
diesel::joinable!(device_tag -> device_descriptor (device_id));
diesel::joinable!(executor_descriptor -> peer_descriptor (peer_id));
diesel::joinable!(executor_kind_container -> executor_descriptor (executor_id));
diesel::joinable!(network_interface_descriptor -> peer_descriptor (peer_id));
diesel::joinable!(network_interface_kind_can -> network_interface_descriptor (network_interface_id));

diesel::allow_tables_to_appear_in_same_query!(
    cluster_configuration,
    cluster_deployment,
    cluster_device,
    device_descriptor,
    device_tag,
    executor_descriptor,
    executor_kind_container,
    network_interface_descriptor,
    network_interface_kind_can,
    peer_descriptor,
);
