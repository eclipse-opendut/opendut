CREATE TABLE cluster_configuration (
    cluster_id uuid PRIMARY KEY,
    name text NOT NULL,
    leader_id uuid NOT NULL REFERENCES peer_descriptor(peer_id) ON DELETE CASCADE,
    deployment_requested bool NOT NULL DEFAULT FALSE
);
CREATE INDEX cluster_configuration_leader_id_index ON cluster_configuration(leader_id);

CREATE TABLE cluster_device (
    cluster_id uuid REFERENCES cluster_configuration(cluster_id) ON DELETE CASCADE,
    device_id uuid REFERENCES device_descriptor(device_id) ON DELETE CASCADE,
    PRIMARY KEY(cluster_id, device_id)
);
