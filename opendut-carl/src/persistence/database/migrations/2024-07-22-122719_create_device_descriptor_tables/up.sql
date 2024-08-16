CREATE TABLE device_descriptor (
    device_id uuid PRIMARY KEY,
    name text NOT NULL,
    description text NULL,
    network_interface_id uuid NULL REFERENCES network_interface_descriptor(network_interface_id) ON DELETE CASCADE
);
CREATE INDEX device_descriptor_network_interface_id_index ON device_descriptor(network_interface_id);

CREATE TABLE device_tag (
    device_id uuid REFERENCES device_descriptor(device_id) ON DELETE CASCADE,
    name text NOT NULL,
    PRIMARY KEY(device_id, name)
);
CREATE INDEX device_tag_name_index ON device_tag(name);
