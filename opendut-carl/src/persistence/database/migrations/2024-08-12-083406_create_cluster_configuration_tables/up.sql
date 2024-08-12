CREATE TABLE cluster_configuration (
    cluster_id uuid PRIMARY KEY,
    name text NOT NULL,
    leader_id uuid NOT NULL REFERENCES peer_descriptor(peer_id) ON DELETE CASCADE
);

CREATE TABLE cluster_device (
    cluster_id uuid REFERENCES cluster_configuration(cluster_id) ON DELETE CASCADE,
    device_id uuid REFERENCES device_descriptor(device_id) ON DELETE CASCADE,
    PRIMARY KEY(cluster_id, device_id)
);
