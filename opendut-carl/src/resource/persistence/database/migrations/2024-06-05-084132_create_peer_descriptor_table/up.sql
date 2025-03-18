CREATE TABLE peer_descriptor (
    peer_id uuid PRIMARY KEY,
    name text NOT NULL,
    location text NULL,
    network_bridge_name text NULL
);
