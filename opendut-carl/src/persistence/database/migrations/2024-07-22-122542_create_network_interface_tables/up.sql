CREATE TYPE network_interface_kind AS ENUM ('ethernet', 'can');

CREATE TABLE network_interface (
    network_interface_id uuid PRIMARY KEY,
    name text NOT NULL,
    kind network_interface_kind NOT NULL,
    peer_id uuid REFERENCES peer_descriptor(peer_id) NOT NULL
);

CREATE TABLE network_interface_kind_can (
    network_interface_id uuid PRIMARY KEY REFERENCES network_interface(network_interface_id),

    bitrate integer NOT NULL,
    sample_point integer NOT NULL,
    fd boolean NOT NULL,
    data_bitrate integer NOT NULL,
    data_sample_point integer NOT NULL
);
