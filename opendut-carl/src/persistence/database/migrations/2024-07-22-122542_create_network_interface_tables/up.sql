CREATE TYPE network_interface_kind AS ENUM ('ethernet', 'can');

CREATE TABLE network_interface_descriptor (
    network_interface_id uuid PRIMARY KEY,
    name text NOT NULL,
    kind network_interface_kind NOT NULL,
    peer_id uuid NOT NULL REFERENCES peer_descriptor(peer_id) ON DELETE CASCADE
);

CREATE TABLE network_interface_kind_can (
    network_interface_id uuid PRIMARY KEY REFERENCES network_interface_descriptor(network_interface_id) ON DELETE CASCADE,

    bitrate integer NOT NULL,
    sample_point_times_1000 integer NOT NULL,
    fd boolean NOT NULL,
    data_bitrate integer NOT NULL,
    data_sample_point_times_1000 integer NOT NULL
);
