CREATE TABLE executor_descriptor (
    executor_id uuid PRIMARY KEY,
    kind text NOT NULL,
    results_url text NULL,
    peer_id uuid NOT NULL REFERENCES peer_descriptor(peer_id) ON DELETE CASCADE
);
CREATE INDEX executor_descriptor_peer_id_index ON executor_descriptor(peer_id);

CREATE TABLE executor_kind_container (
    executor_id uuid PRIMARY KEY REFERENCES executor_descriptor(executor_id) ON DELETE CASCADE,
    engine text NOT NULL,
    name text NULL,
    image text NOT NULL,
    volumes text[] NOT NULL,
    devices text[] NOT NULL,
    envs jsonb[] NOT NULL,
    ports text[] NOT NULL,
    command text NULL,
    args text[] NOT NULL
);
