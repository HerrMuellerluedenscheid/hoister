-- Persisted container state. One row per (user, host, project); each
-- agent heartbeat UPSERTs the row. `services` is the JSON-serialized
-- HashMap<ServiceName, ServiceState> (container inspect + last logs).
CREATE TABLE IF NOT EXISTS container_state (
    user_id      TEXT NOT NULL,
    hostname     TEXT NOT NULL,
    project_name TEXT NOT NULL,
    services     JSONB NOT NULL,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, hostname, project_name)
);

CREATE INDEX IF NOT EXISTS container_state_user_idx ON container_state(user_id);
